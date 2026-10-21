# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Arkham Horror: The Card Game players. Primary situations: solo play, local hotseat, online co-op, and solo with mythos AI automation. Job: play campaigns (Night of the Zealot, The Dunwich Legacy) with a rules-faithful engine, carry progression (XP, trauma, victory, boons/blemishes, scenario carryover) across scenarios, and build decks with ArkhamDB import/export.

## Product Purpose

A standalone digital implementation of Arkham Horror: The Card Game that does not depend on Tabletop Simulator. Provides a rules-faithful engine (campaign progression, investigator turn rotation, card effects, chaos bag, phases), 9 investigators plus expansion player/encounter cards, a full deckbuilder with ArkhamDB-compatible JSON as canonical, configurable taboo list (default v1.10), per-player animation intensity, and a hybrid 2.5D canvas tabletop view plus compact mode. Success means a complete, rules-compliant digital campaign experience playable natively on desktop.

## Positioning

Does not depend on Tabletop Simulator; assets are extracted once from an existing TTS workshop mod into a local, gitignored assets directory. ArkhamDB-compatible JSON is canonical for deck import/export. Card data lives in committed data/ as ArkhamDB-compatible JSON.

## Operating Context

Native desktop application built with Tauri 2. Frontend dev server on http://localhost:5173 (`pnpm dev`); full desktop app via `pnpm tauri:dev`; release bundle via `pnpm tauri:build`. Dedicated multiplayer WebSocket server in server/ (Docker, port 8787). Local save/load state. Prerequisites: Node.js 24+, pnpm 12, Rust stable (MSVC on Windows), Tauri 2 platform prerequisites, Docker only for multiplayer server.

## Capabilities and Constraints

Capabilities: core campaign play (Night of the Zealot, 3 scenarios; The Dunwich Legacy, 4 scenarios); rules compliance (1 clue per successful investigate, damage + horror on failed fight, auto-engage, attacks of opportunity); card effect mechanics (discard, draw, add to play area, gain/spend resources, heal/take damage and horror); investigator turn rotation during Investigation phase; local save/load; local hotseat, online co-op, solo with mythos AI automation; configurable taboo list; full deckbuilder with ArkhamDB import/export; configurable per-player animation intensity; hybrid 2.5D canvas tabletop plus compact mode; optional lazy-loaded Three.js 3D.
Constraints: Node.js >= 24; pnpm 12; Rust stable MSVC toolchain on Windows; Docker only for the multiplayer server; assets never committed (extracted once via `pnpm extract-assets`); cards validated with `pnpm validate-data`.

## Brand Commitments

Name: Arkham Horror: The Card Game — Digital (arkham-horror-lcg). License: MIT.

## Evidence on Hand

README.md (features, tech stack, repo layout, getting started, asset extraction, multiplayer server, data layout, roadmap, docs). data/ (committed ArkhamDB card JSON: investigators, cards, scenarios, tokens). docs/architecture.md. public/ (card images, tokens). resources/ (rulebooks and reference images from TTS mod). src/ (React frontend), src-tauri/ (Rust game engine + Tauri shell), server/ (multiplayer WebSocket server), scripts/ (asset extraction and data validation).

