import { createHash } from 'node:crypto'
import { existsSync } from 'node:fs'
import { copyFile, mkdir, readFile, readdir, stat, writeFile } from 'node:fs/promises'
import { dirname, extname, join, relative, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..')
const MAX_WALK_DEPTH = 64
const IMAGE_EXTENSIONS = ['.png', '.jpg', '.jpeg', '.webp', '.gif', '.bmp']
const MODEL_EXTENSIONS = ['.obj', '.mtl', '.fbx']
const ASSET_HOSTS = ['steamusercontent.com', 'akamaihd.net', 'steamstatic.com']

const LOCAL_SOURCES = [
  { directory: 'Images', kind: 'image', tag: 'images' },
  { directory: 'Images Raw', kind: 'image', tag: 'images-raw' },
  { directory: 'Models', kind: 'model', tag: 'models' },
  { directory: 'Models Raw', kind: 'model', tag: 'models-raw' }
] as const

const HELP = `Usage: extract-assets [options]

Download and copy TTS assets referenced by a workshop mod JSON.

Options:
  --input <path>      Workshop JSON (default $ARKHAM_WORKSHOP_JSON)
  --output <dir>      Output assets dir (default $ARKHAM_ASSETS_DIR or <repo>/assets)
  --tts-root <dir>    Tabletop Simulator data root (default $ARKHAM_TTS_ROOT)
  --concurrency <n>   Parallel downloads (default 8)
  --limit <n>         Process at most n remote assets
  --dry-run           Resolve assets without writing files
  --force             Re-download or re-copy existing files
  --allow-failures    Exit zero even when some downloads fail
  --any-url           Treat every http(s) URL as an asset
  -h, --help          Show this help
`

interface AssetEntry {
  key: string
  kind: 'image' | 'model'
  source: 'remote' | 'local'
  origin: string
  file: string
  bytes: number | null
  sha256: string | null
  contentType: string | null
  status: 'written' | 'skipped' | 'failed' | 'planned'
  error: string | null
}

interface Counts {
  total: number
  written: number
  skipped: number
  failed: number
  planned: number
  images: number
  models: number
}

interface Options {
  input: string
  output: string
  ttsRoot: string
  concurrency: number
  limit: number | null
  dryRun: boolean
  force: boolean
  allowFailures: boolean
  anyUrl: boolean
}

function defaultTtsRoot(): string {
  return (
    process.env.ARKHAM_TTS_ROOT ??
    'C:/Users/Michael/Documents/My Games/Tabletop Simulator'
  )
}

function resolveDefaults(): Options {
  return {
    input:
      process.env.ARKHAM_WORKSHOP_JSON ??
      join(defaultTtsRoot(), 'Mods', 'Workshop', '3776393816.json'),
    output: process.env.ARKHAM_ASSETS_DIR ?? join(REPO_ROOT, 'assets'),
    ttsRoot: defaultTtsRoot(),
    concurrency: 8,
    limit: null,
    dryRun: false,
    force: false,
    allowFailures: false,
    anyUrl: false
  }
}

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

function nextValue(argv: readonly string[], index: number, flag: string): string {
  const value = argv[index + 1]
  if (value === undefined) throw new Error(`missing value for ${flag}`)
  return value
}

function parseInteger(value: string, flag: string): number {
  const parsed = Number.parseInt(value, 10)
  if (!Number.isFinite(parsed) || parsed <= 0) {
    throw new Error(`invalid number for ${flag}: ${value}`)
  }
  return parsed
}

function parseArgs(argv: readonly string[]): Options | null {
  const options = resolveDefaults()
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index]
    if (arg === undefined) continue
    switch (arg) {
      case '-h':
      case '--help':
        process.stdout.write(HELP)
        return null
      case '--input':
        options.input = nextValue(argv, index, arg)
        index += 1
        break
      case '--output':
        options.output = nextValue(argv, index, arg)
        index += 1
        break
      case '--tts-root':
        options.ttsRoot = nextValue(argv, index, arg)
        index += 1
        break
      case '--concurrency':
        options.concurrency = parseInteger(nextValue(argv, index, arg), arg)
        index += 1
        break
      case '--limit':
        options.limit = parseInteger(nextValue(argv, index, arg), arg)
        index += 1
        break
      case '--dry-run':
        options.dryRun = true
        break
      case '--force':
        options.force = true
        break
      case '--allow-failures':
        options.allowFailures = true
        break
      case '--any-url':
        options.anyUrl = true
        break
      default:
        throw new Error(`unknown option: ${arg}`)
    }
  }
  return options
}

function collectStrings(value: unknown, out: string[], depth: number): void {
  if (depth > MAX_WALK_DEPTH) return
  if (typeof value === 'string') {
    out.push(value)
    return
  }
  if (Array.isArray(value)) {
    for (const item of value) collectStrings(item, out, depth + 1)
    return
  }
  if (value !== null && typeof value === 'object') {
    for (const nested of Object.values(value as Record<string, unknown>)) {
      collectStrings(nested, out, depth + 1)
    }
  }
}

function isAssetUrl(value: string, anyUrl: boolean): boolean {
  let parsed: URL
  try {
    parsed = new URL(value)
  } catch {
    return false
  }
  if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') return false
  if (anyUrl) return true
  const host = parsed.hostname.toLowerCase()
  return ASSET_HOSTS.some((suffix) => host === suffix || host.endsWith(`.${suffix}`))
}

function assetKey(prefix: string, value: string): string {
  return `${prefix}_${createHash('sha256').update(value).digest('hex').slice(0, 16)}`
}

function extensionFromUrl(value: string): string {
  let pathname: string
  try {
    pathname = new URL(value).pathname
  } catch {
    return '.bin'
  }
  const extension = extname(pathname).toLowerCase()
  return IMAGE_EXTENSIONS.includes(extension) ? extension : '.bin'
}

async function listFiles(root: string): Promise<string[]> {
  const results: string[] = []
  const walk = async (directory: string): Promise<void> => {
    const entries = await readdir(directory, { withFileTypes: true })
    for (const entry of entries) {
      const full = join(directory, entry.name)
      if (entry.isDirectory()) await walk(full)
      else if (entry.isFile()) results.push(full)
    }
  }
  await walk(root)
  return results.sort()
}

async function mapPool<T, R>(
  items: readonly T[],
  concurrency: number,
  worker: (item: T, index: number) => Promise<R>
): Promise<R[]> {
  const results = new Array<R>(items.length)
  const size = Math.max(1, Math.floor(concurrency))
  let cursor = 0
  const next = async (): Promise<void> => {
    for (;;) {
      const index = cursor
      cursor += 1
      if (index >= items.length) return
      const item = items[index]
      if (item === undefined) continue
      results[index] = await worker(item, index)
    }
  }
  const workers: Promise<void>[] = []
  for (let i = 0; i < size; i += 1) workers.push(next())
  await Promise.all(workers)
  return results
}

async function downloadAsset(
  url: string,
  outputDir: string,
  options: Options
): Promise<AssetEntry> {
  const key = assetKey('img', url)
  const relativeFile = join('images', `${key}${extensionFromUrl(url)}`)
  const destination = join(outputDir, relativeFile)
  const base: AssetEntry = {
    key,
    kind: 'image',
    source: 'remote',
    origin: url,
    file: relativeFile,
    bytes: null,
    sha256: null,
    contentType: null,
    status: 'planned',
    error: null
  }
  if (options.dryRun) return base
  if (!options.force && existsSync(destination)) {
    const info = await stat(destination)
    return { ...base, bytes: info.size, status: 'skipped' }
  }
  try {
    const response = await fetch(url)
    if (!response.ok) {
      return { ...base, status: 'failed', error: `http ${response.status}` }
    }
    const buffer = Buffer.from(await response.arrayBuffer())
    await mkdir(dirname(destination), { recursive: true })
    await writeFile(destination, buffer)
    return {
      ...base,
      bytes: buffer.byteLength,
      sha256: createHash('sha256').update(buffer).digest('hex'),
      contentType: response.headers.get('content-type'),
      status: 'written'
    }
  } catch (error) {
    return { ...base, status: 'failed', error: messageOf(error) }
  }
}

async function copyLocalAssets(
  outputDir: string,
  options: Options
): Promise<AssetEntry[]> {
  const entries: AssetEntry[] = []
  for (const source of LOCAL_SOURCES) {
    const root = join(options.ttsRoot, 'Mods', source.directory)
    if (!existsSync(root)) continue
    const allowed = source.kind === 'image' ? IMAGE_EXTENSIONS : MODEL_EXTENSIONS
    const files = await listFiles(root)
    for (const file of files) {
      const extension = extname(file).toLowerCase()
      if (!allowed.includes(extension)) continue
      const relativeSource = relative(root, file)
      const relativeFile = join(
        source.kind === 'image' ? 'images' : 'models',
        'local',
        source.tag,
        relativeSource
      )
      const destination = join(outputDir, relativeFile)
      const base: AssetEntry = {
        key: assetKey('local', `${source.tag}/${relativeSource}`),
        kind: source.kind,
        source: 'local',
        origin: file,
        file: relativeFile,
        bytes: null,
        sha256: null,
        contentType: null,
        status: 'planned',
        error: null
      }
      if (options.dryRun) {
        entries.push(base)
        continue
      }
      if (!options.force && existsSync(destination)) {
        const info = await stat(destination)
        entries.push({ ...base, bytes: info.size, status: 'skipped' })
        continue
      }
      try {
        await mkdir(dirname(destination), { recursive: true })
        await copyFile(file, destination)
        const info = await stat(destination)
        entries.push({ ...base, bytes: info.size, status: 'written' })
      } catch (error) {
        entries.push({ ...base, status: 'failed', error: messageOf(error) })
      }
    }
  }
  return entries
}

function computeCounts(entries: readonly AssetEntry[]): Counts {
  const counts: Counts = {
    total: entries.length,
    written: 0,
    skipped: 0,
    failed: 0,
    planned: 0,
    images: 0,
    models: 0
  }
  for (const entry of entries) {
    if (entry.kind === 'image') counts.images += 1
    else counts.models += 1
    switch (entry.status) {
      case 'written':
        counts.written += 1
        break
      case 'skipped':
        counts.skipped += 1
        break
      case 'failed':
        counts.failed += 1
        break
      case 'planned':
        counts.planned += 1
        break
    }
  }
  return counts
}

async function writeManifest(
  outputDir: string,
  inputPath: string,
  options: Options,
  entries: readonly AssetEntry[],
  counts: Counts
): Promise<string> {
  const manifestPath = join(outputDir, 'manifest.json')
  const manifest = {
    generatedAt: new Date().toISOString(),
    workshopJson: inputPath,
    ttsRoot: resolve(options.ttsRoot),
    counts,
    entries
  }
  await mkdir(outputDir, { recursive: true })
  await writeFile(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`)
  return manifestPath
}

async function main(): Promise<number> {
  let options: Options | null = null
  try {
    options = parseArgs(process.argv.slice(2))
  } catch (error) {
    process.stderr.write(`${messageOf(error)}\n`)
    return 1
  }
  if (options === null) return 0

  const inputPath = resolve(options.input)
  if (!existsSync(inputPath)) {
    process.stderr.write(`workshop json not found: ${inputPath}\n`)
    return 1
  }

  const outputDir = resolve(options.output)
  const parsed: unknown = JSON.parse(await readFile(inputPath, 'utf8'))

  const strings: string[] = []
  collectStrings(parsed, strings, 0)

  const urls = new Set<string>()
  for (const value of strings) {
    if (isAssetUrl(value, options.anyUrl)) urls.add(value)
  }
  const remoteUrls = [...urls].sort()
  const selected = options.limit === null ? remoteUrls : remoteUrls.slice(0, options.limit)

  const remoteEntries = await mapPool(selected, options.concurrency, (url) =>
    downloadAsset(url, outputDir, options)
  )
  const localEntries = await copyLocalAssets(outputDir, options)
  const entries = [...remoteEntries, ...localEntries]
  const counts = computeCounts(entries)
  const manifestPath = await writeManifest(outputDir, inputPath, options, entries, counts)

  process.stdout.write(
    `assets: ${counts.total} (written ${counts.written}, skipped ${counts.skipped}, failed ${counts.failed}, planned ${counts.planned})\n`
  )
  process.stdout.write(`manifest: ${manifestPath}\n`)

  if (counts.failed > 0 && !options.allowFailures) return 1
  return 0
}

try {
  process.exitCode = await main()
} catch (error) {
  process.stderr.write(`${messageOf(error)}\n`)
  process.exitCode = 1
}
