import { invoke } from "@tauri-apps/api/core";
import type { ChaosToken, GameState, Investigator } from "../types";

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function newGame(
  campaign: string,
  scenario: string,
  seed?: number | null,
): Promise<GameState> {
  return invoke<GameState>("new_game", {
    campaign,
    scenario,
    seed: seed ?? null,
  });
}

export async function addInvestigator(
  investigator: Pick<
    Investigator,
    "code" | "name" | "willpower" | "intellect" | "combat" | "agility" | "max_health" | "max_sanity"
  >,
): Promise<GameState> {
  return invoke<GameState>("add_investigator", {
    code: investigator.code,
    name: investigator.name,
    willpower: investigator.willpower,
    intellect: investigator.intellect,
    combat: investigator.combat,
    agility: investigator.agility,
    max_health: investigator.max_health,
    max_sanity: investigator.max_sanity,
  });
}

export async function getGameState(): Promise<GameState | null> {
  try {
    return await invoke<GameState | null>("get_game_state");
  } catch {
    return null;
  }
}

export async function advancePhase(): Promise<void> {
  await invoke("advance_phase");
}

export async function drawChaosToken(): Promise<ChaosToken> {
  return invoke<ChaosToken>("draw_chaos_token");
}

export async function saveGame(path: string): Promise<void> {
  await invoke("save_game", { path });
}

export async function loadGame(path: string): Promise<GameState> {
  return invoke<GameState>("load_game", { path });
}
