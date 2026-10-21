# Architecture

Arkham Horror LCG is a standalone desktop application. Nothing in the runtime depends on
Tabletop Simulator; TTS is used only as a one-time source of art and model assets.

## Layers

```
┌──────────────────────────────────────────────────────────┐
│ Tauri shell (src-tauri)                                   │
│  ├── Rust game engine (rules, state, chaos bag, phases)   │
│  └── Tauri commands (IPC surface)                         │
├──────────────────────────────────────────────────────────┤
│ React frontend (src)                                      │
│  ├── Zustand stores        local UI + session state       │
│  ├── TanStack Query        server + command data          │
│  ├── Canvas renderer       2.5D tabletop (default)        │
│  └── Optional Three.js     lazily loaded 3D view          │
├──────────────────────────────────────────────────────────┤
│ Multiplayer server (server)                               │
│  └── WebSocket relay — authoritative room/player fan-out  │
└──────────────────────────────────────────────────────────┘
```

## Monorepo layout

| Path         | Package            | Role                                        |
| ------------ | ------------------ | ------------------------------------------- |
| `/`          | `arkham-horror-lcg`| Tauri desktop app (React + `src-tauri`)     |
| `server/`    | `@arkham/server`   | Dedicated multiplayer WebSocket server      |
| `scripts/`   | `@arkham/scripts`  | Asset extraction and data validation tooling|

The root package is implicit in the workspace; `server` and `scripts` are listed in
`pnpm-workspace.yaml`. Each workspace package declares its own devDependencies because pnpm does
not expose the root manifest's devDependencies to sub-packages.

All packages extend `tsconfig.base.json`, which enables strict type checking,
`verbatimModuleSyntax`, `noUncheckedIndexedAccess`, and `noUnusedLocals`/`noUnusedParameters`.

## Frontend

- **State**: Zustand holds the client-side session and UI state. TanStack Query wraps data
  fetched from Tauri commands or the multiplayer server.
- **Rendering**: The tabletop is drawn on a 2.5D canvas by default. The Three.js 3D view is
  code-split and only loaded when a player enables it.
- **Styling**: Tailwind CSS with a small custom `arkham` palette; Radix UI primitives for
  accessible dialogs, menus, and tabs; Framer Motion for transitions.
- **IPC**: Frontend calls the Rust engine through Tauri `invoke` wrappers in `src/lib/api.ts`.

## Engine (Rust)

The engine owns all rules and game state and is the single source of truth during local play.

```
engine/
├── mod.rs           module wiring + public re-exports
├── card.rs          Card, CardId, CardType, CardClass, SkillIcons
├── chaos.rs         ChaosToken, ChaosBag (deterministic seeded draws)
├── investigator.rs  Investigator, Deck, DeckSlot
├── phase.rs         Phase enum + transitions
└── game_state.rs    GameState — rounds, phases, log, encounter deck, campaign progress
```

Key properties:

- **Deterministic**: the chaos bag uses a seeded `ChaCha20Rng` derived from `seed + draws`, so a
  game is reproducible from its seed.
- **Errors**: a single `GameError` enum with a serializable representation crosses the IPC
  boundary.
- **Commands**: registered in `src-tauri/src/lib.rs` and backed by `AppState`
  (`Mutex<Option<GameState>>`). Includes `new_game`, `get_game_state`, `advance_phase`,
  `draw_chaos_token`, `next_investigator`, `move_to_location`, `investigate`, `fight`,
  `engage`, `play_card`, `discard_card`, `draw_card`, `add_card_to_play_area`,
  `gain_resource`, `spend_resources`, `heal_damage`, `heal_horror`, `take_damage`,
  `take_horror`, `place_doom`, `advance_act`, `save_game`, `load_game`,
  `add_campaign_log_entry`, `start_next_scenario`, and `open_pdf`.

### Campaign progression

The engine models campaign-wide state through `CampaignProgress` (`completed_scenarios`,
`campaign_log`, `boons_and_blemishes`) carried on `GameState`. Investigators track
`physical_trauma`, `mental_trauma`, `xp`, `is_killed`, and `is_driven_insane` across
scenarios. `start_scenario_in_campaign` carries XP, decklist, and trauma forward while
resetting scenario-specific state; resources reset to 5 and trauma damage/horror are applied
at setup (per the Rules Reference, page 5).

## Data and saves

- **Cards**: ArkhamDB-compatible JSON in `data/cards/`, validated by the `validate-data` script.
- **Investigators**: `data/investigators/` — 9 investigators across core + Dunwich Legacy.
- **Scenarios**: `data/scenarios/` — 7 scenarios across 2 campaigns (*Night of the Zealot*,
  *The Dunwich Legacy*).
- **Tokens**: `data/tokens/` — chaos bag and game token definitions.
- **Saves**: local files written by the Rust engine via `save_game`/`load_game`, with an
  optional encrypted cloud sync path.

## Testing

- Rust E2E test suite: `src-tauri/tests/e2e_game_flow.rs` — 20 tests covering full scenario
  playthroughs for all 7 scenarios, campaign progression carryover, win/loss conditions,
  clue pooling, and chaos bag determinism.
- Run with: `cargo test --test e2e_game_flow` (in `src-tauri/`).

## Multiplayer server

`server/src/index.ts` is a dependency-light WebSocket relay:

- assigns each connection a `clientId` and players a display name
- manages room membership by six-character room code
- fans out `room_state`, `player_joined`, `player_left`, and relayed `action` messages
- keeps connections alive with a 30-second heartbeat
- serves an HTTP health endpoint reporting room and client counts

Client and server messages are discriminated unions (`ClientMessage`, `ServerMessage`). The
server relays actions; rules remain in the engine.

## Asset pipeline

1. `scripts/src/extract-assets.ts` reads the TTS workshop JSON, walks it for asset URLs, and
   downloads images and models into `assets/`, also copying local TTS asset files. It writes
   `assets/manifest.json` with per-file digests.
2. `scripts/src/build-atlases.ts` groups extracted sprites into an atlas layout for the renderer.
3. `scripts/src/validate-data.ts` validates the committed ArkhamDB card JSON in `data/`.

Assets are never committed; `data/` JSON is committed and treated as source of truth.

## Taboo list

Taboo handling is configurable and defaults to v1.10. The taboo set is applied at deck
construction and validation time.

## Build and release

- Frontend/desktop release bundles are produced by `corepack pnpm tauri:build`.
- The multiplayer server ships as a Docker image; `server/fly.toml` describes the Fly.io
  deployment.
