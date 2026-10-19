import { existsSync } from 'node:fs';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { packImages, type LoadedImage, type Page, type SkippedImage } from './lib/pack.js';
import sharp from 'sharp';

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..');
const DEFAULT_ATLAS_SIZE = 2048;
const MIN_ATLAS_SIZE = 64;
const MAX_ATLAS_SIZE = 8192;
const DEFAULT_PADDING = 2;
const MAX_PADDING = 64;
const DEFAULT_CONCURRENCY = 8;

const HELP = `Arkham Horror LCG atlas builder

Usage:
  tsx src/build-atlases.ts [options]

Options:
  --input <path>     Asset manifest produced by extract-assets (default: assets/manifest.json)
  --output <dir>     Directory for packed atlases (default: assets/atlases)
  --size <px>        Maximum atlas page size in pixels (default: 2048)
  --padding <px>     Space between packed images in pixels (default: 2)
  --limit <count>    Only process the first N images
  --dry-run          Plan atlas pages without writing files
  --force            Rebuild atlases even when metadata already exists
  -h, --help         Show this message

Environment:
  ARKHAM_MANIFEST    Override the input manifest path
  ARKHAM_ATLAS_DIR   Override the output directory
`;

interface ManifestEntry {
  key: string;
  kind: 'image' | 'model';
  source: 'remote' | 'local';
  origin: string;
  file: string;
  bytes: number | null;
  sha256: string | null;
  contentType: string | null;
  status: 'written' | 'skipped' | 'failed' | 'planned';
  error: string | null;
}

interface Manifest {
  generatedAt: string;
  workshopJson: string;
  ttsRoot: string;
  counts: Record<string, number>;
  entries: ManifestEntry[];
}

interface Options {
  input: string;
  output: string;
  size: number;
  padding: number;
  limit: number | null;
  dryRun: boolean;
  force: boolean;
}

interface AtlasRecord {
  page: number;
  file: string;
  width: number;
  height: number;
  regions: number;
}

interface AtlasMetadata {
  generatedAt: string;
  source: string;
  size: number;
  padding: number;
  counts: {
    images: number;
    packed: number;
    skipped: number;
    pages: number;
  };
  atlases: AtlasRecord[];
  regions: Record<string, { atlas: string; x: number; y: number; width: number; height: number }>;
}

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function nextValue(argv: string[], index: number, flag: string): string {
  const value = argv[index + 1];
  if (value === undefined) {
    throw new Error(`missing value for ${flag}`);
  }
  return value;
}

function parseInteger(value: string, flag: string): number {
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed)) {
    throw new Error(`invalid number for ${flag}: ${value}`);
  }
  return parsed;
}

function parseArgs(argv: string[]): Options | null {
  const options: Options = {
    input: process.env.ARKHAM_MANIFEST ?? join(REPO_ROOT, 'assets', 'manifest.json'),
    output: process.env.ARKHAM_ATLAS_DIR ?? join(REPO_ROOT, 'assets', 'atlases'),
    size: DEFAULT_ATLAS_SIZE,
    padding: DEFAULT_PADDING,
    limit: null,
    dryRun: false,
    force: false,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === undefined) {
      continue;
    }
    switch (arg) {
      case '-h':
      case '--help':
        process.stdout.write(HELP);
        return null;
      case '--input':
        options.input = nextValue(argv, index, arg);
        index += 1;
        break;
      case '--output':
        options.output = nextValue(argv, index, arg);
        index += 1;
        break;
      case '--size': {
        const size = parseInteger(nextValue(argv, index, arg), arg);
        if (size < MIN_ATLAS_SIZE || size > MAX_ATLAS_SIZE) {
          throw new Error(`--size must be between ${MIN_ATLAS_SIZE} and ${MAX_ATLAS_SIZE}`);
        }
        options.size = size;
        index += 1;
        break;
      }
      case '--padding': {
        const padding = parseInteger(nextValue(argv, index, arg), arg);
        if (padding < 0 || padding > MAX_PADDING) {
          throw new Error(`--padding must be between 0 and ${MAX_PADDING}`);
        }
        options.padding = padding;
        index += 1;
        break;
      }
      case '--limit':
        options.limit = parseInteger(nextValue(argv, index, arg), arg);
        index += 1;
        break;
      case '--dry-run':
        options.dryRun = true;
        break;
      case '--force':
        options.force = true;
        break;
      default:
        throw new Error(`unknown option: ${arg}`);
    }
  }

  return options;
}

async function mapPool<T, R>(
  items: T[],
  concurrency: number,
  worker: (item: T, index: number) => Promise<R>,
): Promise<R[]> {
  const results = new Array<R>(items.length);
  let cursor = 0;
  const runners = Array.from(
    { length: Math.max(1, Math.min(concurrency, items.length)) },
    async () => {
      while (true) {
        const index = cursor;
        cursor += 1;
        if (index >= items.length) {
          return;
        }
        const item = items[index];
        if (item === undefined) {
          continue;
        }
        results[index] = await worker(item, index);
      }
    },
  );
  await Promise.all(runners);
  return results;
}

async function loadImage(entry: ManifestEntry, manifestDirectory: string): Promise<LoadedImage> {
  const sourcePath = resolve(manifestDirectory, entry.file);
  if (!existsSync(sourcePath)) {
    throw new Error(`missing file for ${entry.key}: ${sourcePath}`);
  }
  const metadata = await sharp(sourcePath).metadata();
  const width = metadata.width ?? 0;
  const height = metadata.height ?? 0;
  if (width <= 0 || height <= 0) {
    throw new Error(`unreadable dimensions for ${entry.key}`);
  }
  return { key: entry.key, file: entry.file, sourcePath, width, height };
}

async function writePage(page: Page, outputDirectory: string, force: boolean): Promise<AtlasRecord> {
  const file = `atlas-${String(page.page).padStart(3, '0')}.png`;
  const destination = join(outputDirectory, file);
  if (force || !existsSync(destination)) {
    await sharp({
      create: {
        width: page.width,
        height: page.height,
        channels: 4,
        background: { r: 0, g: 0, b: 0, alpha: 0 },
      },
    })
      .composite(page.regions.map((region) => ({ input: region.sourcePath, left: region.x, top: region.y })))
      .png({ compressionLevel: 9 })
      .toFile(destination);
  }
  return { page: page.page, file, width: page.width, height: page.height, regions: page.regions.length };
}

async function main(): Promise<number> {
  const options = parseArgs(process.argv.slice(2));
  if (options === null) {
    return 0;
  }
  const manifestPath = resolve(options.input);
  if (!existsSync(manifestPath)) {
    process.stderr.write(`manifest not found: ${manifestPath}\n`);
    return 1;
  }
  let manifest: Manifest;
  try {
    manifest = JSON.parse(await readFile(manifestPath, 'utf8')) as Manifest;
  } catch (error) {
    process.stderr.write(`invalid manifest: ${manifestPath}: ${messageOf(error)}\n`);
    return 1;
  }
  if (!Array.isArray(manifest.entries)) {
    process.stderr.write(`invalid manifest: ${manifestPath}\n`);
    return 1;
  }
  const metadataPath = join(options.output, 'atlases.json');
  if (!options.force && existsSync(metadataPath)) {
    process.stdout.write(`atlases already exist: ${metadataPath}\n`);
    process.stdout.write('pass --force to rebuild\n');
    return 0;
  }
  const candidates = manifest.entries.filter(
    (entry) => entry.kind === 'image' && (entry.status === 'written' || entry.status === 'skipped'),
  );
  const limited = options.limit === null ? candidates : candidates.slice(0, options.limit);
  const manifestDirectory = dirname(manifestPath);
  const loaded: LoadedImage[] = [];
  const unavailable: SkippedImage[] = [];
  await mapPool(limited, DEFAULT_CONCURRENCY, async (entry) => {
    try {
      loaded.push(await loadImage(entry, manifestDirectory));
    } catch (error) {
      unavailable.push({ key: entry.key, file: entry.file, width: 0, height: 0, reason: messageOf(error) });
    }
  });
  const packed = packImages(loaded, options.size, options.padding);
  if (options.dryRun) {
    process.stdout.write(`dry run: ${packed.pages.length} atlas pages for ${loaded.length} images\n`);
    process.stdout.write(`skipped: ${packed.skipped.length + unavailable.length}\n`);
    return 0;
  }
  await mkdir(options.output, { recursive: true });
  const atlases: AtlasRecord[] = [];
  const regions: Record<string, { atlas: string; x: number; y: number; width: number; height: number }> = {};
  for (const page of packed.pages) {
    const record = await writePage(page, options.output, options.force);
    atlases.push(record);
    for (const region of page.regions) {
      regions[region.key] = {
        atlas: record.file,
        x: region.x,
        y: region.y,
        width: region.width,
        height: region.height,
      };
    }
  }
  const metadata: AtlasMetadata = {
    generatedAt: new Date().toISOString(),
    source: manifestPath,
    size: options.size,
    padding: options.padding,
    counts: {
      images: loaded.length,
      packed: packed.pages.reduce((total, page) => total + page.regions.length, 0),
      skipped: packed.skipped.length + unavailable.length,
      pages: packed.pages.length,
    },
    atlases,
    regions,
  };
  await writeFile(metadataPath, `${JSON.stringify(metadata, null, 2)}\n`, 'utf8');
  process.stdout.write(`packed ${metadata.counts.packed} images into ${metadata.counts.pages} atlas pages\n`);
  process.stdout.write(`skipped: ${metadata.counts.skipped}\n`);
  process.stdout.write(`unavailable: ${unavailable.length}\n`);
  process.stdout.write(`metadata: ${metadataPath}\n`);
  return 0;
}

try {
  process.exitCode = await main();
} catch (error) {
  process.stderr.write(`${messageOf(error)}\n`);
  process.exitCode = 1;
}
