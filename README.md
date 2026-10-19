# Arkham Horror: The Card Game — Digital

A standalone digital implementation of *Arkham Horror: The Card Game*, built as a native
desktop application. It does not depend on Tabletop Simulator; assets are extracted once from
an existing TTS workshop mod into a local, gitignored `assets/` directory.

## Features (planned)

- Core campaign play — *Night of the Zealot* (3 scenarios), 5 investigators
- Rules-faithful engine with a manual override escape hatch
- Configurable taboo list (default: v1.10)
- Full deckbuilder with ArkhamDB import/export (ArkhamDB-compatible JSON is canonical)
- Local hotseat, online co-op, and solo with mythos AI automation
- Local save files with optional encrypted cloud sync
- Configurable per-player animation intensity
- Hybrid UI: 2.5D canvas tabletop view plus a compact mode

## Tech stack

| Layer      | Technology                                              |
| ---------- | ------------------------------------------------------- |
| Desktop    | Tauri 2                                                  |
| Frontend   | React 18, TypeScript, Vite                               |
| Styling    | Tailwind CSS, Radix UI, Framer Motion                    |
| State      | Zustand, TanStack Query                                  |
| Rendering  | 2.5D Canvas (default), optional lazy-loaded Three.js 3D  |
| Engine     | Rust (game state, rules, chaos bag, phases)              |
| Server     | Node.js WebSocket relay (multiplayer)                    |
| Packaging  | Docker, Fly.io                                           |

## Repository layout

```
.
├── src/                 React frontend
├── src-tauri/           Rust game engine + Tauri shell
├── server/              Dedicated multiplayer WebSocket server
├── scripts/             Asset extraction and data validation tooling
├── data/                Committed ArkhamDB card JSON
├── assets/              Extracted assets (gitignored, generated)
└── docs/                Architecture and process documentation
```

The root `package.json` is the Tauri desktop application. `server/` and `scripts/` are separate
pnpm workspace packages.

## Prerequisites

- Node.js 24 or newer
- pnpm 12 (via corepack: `corepack enable pnpm`, or use `corepack pnpm ...`)
- Rust stable (MSVC toolchain on Windows)
- Platform prerequisites for Tauri 2
- Docker (only for the multiplayer server)

## Getting started

```sh
corepack pnpm install
corepack pnpm dev            # browser dev server on http://localhost:5173
corepack pnpm tauri:dev      # full desktop app
```

## Common scripts

| Command                      | Description                                  |
| ---------------------------- | -------------------------------------------- |
| `corepack pnpm dev`          | Vite dev server                              |
| `corepack pnpm build`        | Type-check and build the frontend            |
| `corepack pnpm tauri:dev`    | Run the desktop app in development           |
| `corepack pnpm tauri:build`  | Produce a release desktop bundle             |
| `corepack pnpm typecheck`    | Type-check the frontend                      |
| `corepack pnpm lint`         | Lint the frontend                            |
| `corepack pnpm test`         | Run frontend tests                            |
| `corepack pnpm extract-assets` | Extract assets from the TTS mod            |
| `corepack pnpm validate-data`  | Validate committed card JSON               |

## Asset extraction

Assets are never committed. Extract them once from your local TTS install:

```sh
corepack pnpm extract-assets
```

Defaults can be overridden with environment variables:

- `ARKHAM_TTS_ROOT` — Tabletop Simulator data directory
- `ARKHAM_WORKSHOP_JSON` — path to the workshop mod JSON
- `ARKHAM_ASSETS_DIR` — output directory (default `assets/`)

A `manifest.json` is written alongside the extracted files.

## Multiplayer server

```sh
cd server
docker compose up --build
```

The relay listens on port `8787` and exposes a health endpoint at `/`.

## Data

Card data lives in `data/` as ArkhamDB-compatible JSON and is committed to the repository.
Validate it with `corepack pnpm validate-data` (`--strict` to fail on warnings).

## Roadmap

1. Core campaign fully playable
2. Deckbuilder
3. Multiplayer
4. Expansions

## Documentation

- [Architecture](docs/architecture.md)

## License

MIT
