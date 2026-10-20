import { invoke } from "@tauri-apps/api/core";
import type { CardTemplate } from "../data/cards";
import type {
  AdvanceActResult,
  ChaosToken,
  EngageResult,
  FightResult,
  GameState,
  Investigator,
  InvestigateResult,
  MoveResult,
  MythosResult,
  PlaceDoomResult,
  PlayCardResult,
  SkillTestResult,
} from "../types";

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function newGame(
  campaign: string,
  scenario: string,
  seed?: number | null,
  encounterCards: CardTemplate[] = [],
): Promise<GameState> {
  return invoke<GameState>("new_game", {
    campaign,
    scenario,
    seed: seed ?? null,
    encounter_cards: encounterCards,
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

export async function skillTest(
  investigatorId: string,
  skill: string,
): Promise<SkillTestResult> {
  return invoke<SkillTestResult>("skill_test", {
    investigator_id: investigatorId,
    skill,
  });
}

export async function spendAction(investigatorId: string): Promise<GameState> {
  return invoke<GameState>("spend_action", {
    investigator_id: investigatorId,
  });
}

export async function resetTurn(): Promise<GameState> {
  return invoke<GameState>("reset_turn");
}

export async function mythosPhase(): Promise<MythosResult> {
  return invoke<MythosResult>("mythos_phase");
}

export async function playCard(investigatorId: string, cardId: string): Promise<PlayCardResult> {
  return invoke<PlayCardResult>("play_card", {
    investigator_id: investigatorId,
    card_id: cardId,
  });
}

export async function engage(
  investigatorId: string,
  enemyInstanceId: string,
): Promise<EngageResult> {
  return invoke<EngageResult>("engage", {
    investigator_id: investigatorId,
    enemy_instance_id: enemyInstanceId,
  });
}

export async function fight(
  investigatorId: string,
  enemyInstanceId: string,
): Promise<FightResult> {
  return invoke<FightResult>("fight", {
    investigator_id: investigatorId,
    enemy_instance_id: enemyInstanceId,
  });
}

export async function moveToLocation(
  investigatorId: string,
  locationInstanceId: string,
): Promise<MoveResult> {
  return invoke<MoveResult>("move_to_location", {
    investigator_id: investigatorId,
    location_instance_id: locationInstanceId,
  });
}

export async function investigate(investigatorId: string): Promise<InvestigateResult> {
  return invoke<InvestigateResult>("investigate", {
    investigator_id: investigatorId,
  });
}

export async function placeDoom(amount: number): Promise<PlaceDoomResult> {
  return invoke<PlaceDoomResult>("place_doom", { amount });
}

export async function advanceAct(
  investigatorId: string,
  cluesSpent: number,
): Promise<AdvanceActResult> {
  return invoke<AdvanceActResult>("advance_act", {
    investigator_id: investigatorId,
    clues_spent: cluesSpent,
  });
}

export async function saveGame(path: string): Promise<void> {
  await invoke("save_game", { path });
}

export async function loadGame(path: string): Promise<GameState> {
  return invoke<GameState>("load_game", { path });
}
