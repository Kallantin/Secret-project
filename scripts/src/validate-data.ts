import { existsSync } from 'node:fs'
import { readFile, readdir } from 'node:fs/promises'
import { dirname, extname, join, relative, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..')
const MAX_ISSUE_DEFAULT = 200

const CARD_TYPE_CODES = new Set([
  'act',
  'agenda',
  'asset',
  'enemy',
  'event',
  'investigator',
  'location',
  'skill',
  'story',
  'treachery',
  'weakness'
])

const FACTION_CODES = new Set([
  'guardian',
  'mythos',
  'neutral',
  'rogue',
  'seeker',
  'survivor',
  'mystic'
])

const CARD_COLLECTION_KEYS = ['cards', 'data', 'items']

const NUMBER_FIELDS = [
  'cost',
  'xp',
  'health',
  'sanity',
  'willpower',
  'intellect',
  'combat',
  'agility',
  'position',
  'quantity',
  'deck_limit'
]

const STRING_FIELDS = [
  'name',
  'text',
  'subname',
  'flavor',
  'slot',
  'illustrator',
  'pack_code',
  'type_code',
  'faction_code',
  'real_name',
  'real_text'
]

const HELP = `Usage: validate-data [options]

Validate ArkhamDB-style card JSON under the data directory.

Options:
  --data <dir>        Data directory (default $ARKHAM_DATA_DIR or <repo>/data)
  --json              Emit machine-readable JSON results
  --strict            Treat warnings and a missing data directory as errors
  --max-issues <n>    Report at most n issues (default ${MAX_ISSUE_DEFAULT})
  -h, --help          Show this help
`

interface Issue {
  file: string
  path: string
  message: string
  severity: 'error' | 'warning'
}

interface ReportState {
  issues: Issue[]
  errors: number
  warnings: number
}

interface Options {
  data: string
  json: boolean
  strict: boolean
  maxIssues: number
}

interface CheckedCounts {
  cards: number
  files: number
  skippedFiles: number
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
  const options: Options = {
    data: process.env.ARKHAM_DATA_DIR ?? join(REPO_ROOT, 'data'),
    json: false,
    strict: false,
    maxIssues: MAX_ISSUE_DEFAULT
  }
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index]
    if (arg === undefined) continue
    switch (arg) {
      case '-h':
      case '--help':
        process.stdout.write(HELP)
        return null
      case '--data':
        options.data = nextValue(argv, index, arg)
        index += 1
        break
      case '--json':
        options.json = true
        break
      case '--strict':
        options.strict = true
        break
      case '--max-issues':
        options.maxIssues = parseInteger(nextValue(argv, index, arg), arg)
        index += 1
        break
      default:
        throw new Error(`unknown option: ${arg}`)
    }
  }
  return options
}

async function findJsonFiles(root: string): Promise<string[]> {
  const names = await readdir(root, { recursive: true })
  return names
    .filter((name) => extname(name).toLowerCase() === '.json')
    .map((name) => join(root, name))
    .sort()
}

function looksLikeCard(record: Record<string, unknown>): boolean {
  const code = record['code']
  const typeCode = record['type_code']
  return (
    typeof code === 'string' &&
    code.length > 0 &&
    typeof typeCode === 'string' &&
    typeCode.length > 0
  )
}

function extractCards(parsed: unknown): unknown[] | null {
  if (Array.isArray(parsed)) {
    const first = parsed[0]
    if (
      first !== null &&
      typeof first === 'object' &&
      !Array.isArray(first) &&
      looksLikeCard(first as Record<string, unknown>)
    ) {
      return parsed
    }
    return null
  }
  if (parsed !== null && typeof parsed === 'object') {
    const record = parsed as Record<string, unknown>
    for (const key of CARD_COLLECTION_KEYS) {
      const value = record[key]
      if (!Array.isArray(value)) continue
      const first = value[0]
      if (
        first !== null &&
        typeof first === 'object' &&
        !Array.isArray(first) &&
        looksLikeCard(first as Record<string, unknown>)
      ) {
        return value
      }
    }
    if (looksLikeCard(record)) return [record]
    return null
  }
  return null
}

function pushIssue(state: ReportState, options: Options, issue: Issue): void {
  if (issue.severity === 'error') state.errors += 1
  else state.warnings += 1
  if (state.issues.length < options.maxIssues) state.issues.push(issue)
}

function validateCard(
  card: Record<string, unknown>,
  file: string,
  index: number,
  seenCodes: Map<string, string>,
  state: ReportState,
  options: Options
): void {
  const path = `$[${index}]`

  const code = card['code']
  if (typeof code !== 'string' || code.length === 0) {
    pushIssue(state, options, {
      file,
      path: `${path}.code`,
      message: 'missing or empty code',
      severity: 'error'
    })
  } else {
    const existing = seenCodes.get(code)
    if (existing !== undefined) {
      pushIssue(state, options, {
        file,
        path: `${path}.code`,
        message: `duplicate code ${code} (also in ${existing})`,
        severity: 'error'
      })
    } else {
      seenCodes.set(code, file)
    }
  }

  const name = card['name']
  if (typeof name !== 'string' || name.length === 0) {
    pushIssue(state, options, {
      file,
      path: `${path}.name`,
      message: 'missing or empty name',
      severity: 'error'
    })
  }

  const typeCode = card['type_code']
  if (typeof typeCode !== 'string' || typeCode.length === 0) {
    pushIssue(state, options, {
      file,
      path: `${path}.type_code`,
      message: 'missing type_code',
      severity: 'error'
    })
  } else if (!CARD_TYPE_CODES.has(typeCode)) {
    pushIssue(state, options, {
      file,
      path: `${path}.type_code`,
      message: `unknown type_code: ${typeCode}`,
      severity: 'warning'
    })
  }

  const factionCode = card['faction_code']
  if (typeof factionCode !== 'string' || factionCode.length === 0) {
    pushIssue(state, options, {
      file,
      path: `${path}.faction_code`,
      message: 'missing faction_code',
      severity: 'error'
    })
  } else if (!FACTION_CODES.has(factionCode)) {
    pushIssue(state, options, {
      file,
      path: `${path}.faction_code`,
      message: `unknown faction_code: ${factionCode}`,
      severity: 'warning'
    })
  }

  for (const field of NUMBER_FIELDS) {
    const value = card[field]
    if (value === undefined || value === null) continue
    if (typeof value !== 'number' || !Number.isFinite(value)) {
      pushIssue(state, options, {
        file,
        path: `${path}.${field}`,
        message: `expected number or null for ${field}`,
        severity: 'error'
      })
    }
  }

  for (const field of STRING_FIELDS) {
    const value = card[field]
    if (value === undefined || value === null) continue
    if (typeof value !== 'string') {
      pushIssue(state, options, {
        file,
        path: `${path}.${field}`,
        message: `expected string or null for ${field}`,
        severity: 'error'
      })
    }
  }
}

function report(state: ReportState, options: Options, counts: CheckedCounts): void {
  if (options.json) {
    process.stdout.write(
      `${JSON.stringify(
        {
          ...counts,
          errors: state.errors,
          warnings: state.warnings,
          issues: state.issues
        },
        null,
        2
      )}\n`
    )
    return
  }
  for (const issue of state.issues) {
    process.stdout.write(
      `${issue.severity.toUpperCase()} ${issue.file} ${issue.path}: ${issue.message}\n`
    )
  }
  process.stdout.write(
    `checked ${counts.cards} cards in ${counts.files} files (${counts.skippedFiles} skipped, ${state.errors} errors, ${state.warnings} warnings)\n`
  )
  if (
    state.issues.length >= options.maxIssues &&
    state.errors + state.warnings > state.issues.length
  ) {
    process.stdout.write(`output truncated at ${options.maxIssues} issues\n`)
  }
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

  const dataDir = resolve(options.data)
  if (!existsSync(dataDir)) {
    process.stderr.write(`data directory not found: ${dataDir}\n`)
    return options.strict ? 1 : 0
  }

  const files = await findJsonFiles(dataDir)
  const state: ReportState = { issues: [], errors: 0, warnings: 0 }
  const seenCodes = new Map<string, string>()
  let cardCount = 0
  let skippedFiles = 0

  for (const file of files) {
    const fileLabel = relative(dataDir, file)
    let parsed: unknown
    try {
      parsed = JSON.parse(await readFile(file, 'utf8'))
    } catch (error) {
      pushIssue(state, options, {
        file: fileLabel,
        path: '$',
        message: `invalid json: ${messageOf(error)}`,
        severity: 'error'
      })
      continue
    }
    const cards = extractCards(parsed)
    if (cards === null) {
      skippedFiles += 1
      continue
    }
    for (let index = 0; index < cards.length; index += 1) {
      const card = cards[index]
      if (card === null || typeof card !== 'object' || Array.isArray(card)) {
        pushIssue(state, options, {
          file: fileLabel,
          path: `$[${index}]`,
          message: 'card is not an object',
          severity: 'error'
        })
        continue
      }
      cardCount += 1
      validateCard(card as Record<string, unknown>, fileLabel, index, seenCodes, state, options)
    }
  }

  report(state, options, { cards: cardCount, files: files.length, skippedFiles })

  if (state.errors > 0) return 1
  if (options.strict && state.warnings > 0) return 1
  return 0
}

try {
  process.exitCode = await main()
} catch (error) {
  process.stderr.write(`${messageOf(error)}\n`)
  process.exitCode = 1
}
