import { invoke } from "@tauri-apps/api/core";
import type { CardTemplate } from "../data/cards";
import type {
  AdvanceActResult,
  CampaignProgress,
  Card,
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
  ScenarioData,
  SkillTestResult,
} from "../types";

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

// Commands that take a single `input` struct parameter (wrap under `input` key)
// Struct field names are preserved as-is (snake_case from Rust).
export async function newGame(
  campaign: string,
  scenario: string,
  seed?: number | null,
  encounterCards: CardTemplate[] = [],
  scenarioData?: ScenarioData,
  campaignProgress?: CampaignProgress | null,
): Promise<GameState> {
  return invoke<GameState>("new_game", {
    input: {
      campaign,
      scenario,
      seed: seed ?? null,
      encounter_cards: encounterCards,
      scenario_data: scenarioData,
      campaign_progress: campaignProgress ?? null,
    },
  });
}

export async function startNextScenario(
  scenario: string,
  seed?: number | null,
  scenarioData?: ScenarioData,
): Promise<GameState> {
  return invoke<GameState>("start_next_scenario", {
    input: { scenario, seed: seed ?? null, scenario_data: scenarioData },
  });
}

export async function addCampaignLogEntry(
  result: string,
  notes: string,
): Promise<GameState> {
  return invoke<GameState>("add_campaign_log_entry", {
    input: { result, notes },
  });
}

export async function advanceAct(
  contributions: Array<{ investigatorId: string; cluesSpent: number }>,
): Promise<AdvanceActResult> {
  return invoke<AdvanceActResult>("advance_act", {
    input: {
      contributions: contributions.map((c) => ({
        investigator_id: c.investigatorId,
        clues_spent: c.cluesSpent,
      })),
    },
  });
}

export async function discardCard(
  investigatorId: string,
  cardId: string,
): Promise<Card> {
  return invoke<Card>("discard_card", {
    input: { investigator_id: investigatorId, card_id: cardId },
  });
}

export async function drawCard(investigatorId: string): Promise<Card> {
  return invoke<Card>("draw_card", { input: { investigator_id: investigatorId } });
}

export async function addCardToPlayArea(
  investigatorId: string,
  card: Card,
): Promise<void> {
  return invoke<void>("add_card_to_play_area", {
    input: { investigator_id: investigatorId, card },
  });
}

export async function gainResource(
  investigatorId: string,
  amount: number,
): Promise<number> {
  return invoke<number>("gain_resource", {
    input: { investigator_id: investigatorId, amount },
  });
}

export async function spendResources(
  investigatorId: string,
  amount: number,
): Promise<void> {
  return invoke<void>("spend_resources_cmd", {
    input: { investigator_id: investigatorId, amount },
  });
}

export async function healDamage(
  investigatorId: string,
  amount: number,
): Promise<number> {
  return invoke<number>("heal_damage_cmd", {
    input: { investigator_id: investigatorId, amount },
  });
}

export async function healHorror(
  investigatorId: string,
  amount: number,
): Promise<number> {
  return invoke<number>("heal_horror_cmd", {
    input: { investigator_id: investigatorId, amount },
  });
}

export async function takeDamage(
  investigatorId: string,
  amount: number,
): Promise<number> {
  return invoke<number>("take_damage_cmd", {
    input: { investigator_id: investigatorId, amount },
  });
}

export async function takeHorror(
  investigatorId: string,
  amount: number,
): Promise<number> {
  return invoke<number>("take_horror_cmd", {
    input: { investigator_id: investigatorId, amount },
  });
}

// Commands that take individual flat parameters.
// Tauri converts Rust snake_case param names to camelCase on the JS side.
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
    maxHealth: investigator.max_health,
    maxSanity: investigator.max_sanity,
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
  return invoke("advance_phase");
}

export async function drawChaosToken(): Promise<ChaosToken> {
  return invoke<ChaosToken>("draw_chaos_token");
}

export async function skillTest(
  investigatorId: string,
  skill: string,
): Promise<SkillTestResult> {
  return invoke<SkillTestResult>("skill_test", {
    investigatorId,
    skill,
  });
}

export async function spendAction(investigatorId: string): Promise<GameState> {
  return invoke<GameState>("spend_action", {
    investigatorId,
  });
}

export async function resetTurn(): Promise<void> {
  return invoke("reset_turn");
}

export async function mythosPhase(): Promise<MythosResult> {
  return invoke<MythosResult>("mythos_phase");
}

export async function playCard(investigatorId: string, cardId: string): Promise<PlayCardResult> {
  return invoke<PlayCardResult>("play_card", {
    investigatorId,
    cardId,
  });
}

export async function engage(
  investigatorId: string,
  enemyInstanceId: string,
): Promise<EngageResult> {
  return invoke<EngageResult>("engage", {
    investigatorId,
    enemyInstanceId,
  });
}

export async function fight(
  investigatorId: string,
  enemyInstanceId: string,
): Promise<FightResult> {
  return invoke<FightResult>("fight", {
    investigatorId,
    enemyInstanceId,
  });
}

export async function moveToLocation(
  investigatorId: string,
  locationInstanceId: string,
): Promise<MoveResult> {
  return invoke<MoveResult>("move_to_location", {
    investigatorId,
    locationInstanceId,
  });
}

export async function investigate(investigatorId: string): Promise<InvestigateResult> {
  return invoke<InvestigateResult>("investigate", {
    investigatorId,
  });
}

export async function placeDoom(amount: number): Promise<PlaceDoomResult> {
  return invoke<PlaceDoomResult>("place_doom", { amount });
}

export async function saveGame(path: string): Promise<void> {
  await invoke("save_game", { path });
}

export async function loadGame(path: string): Promise<GameState> {
  return invoke<GameState>("load_game", { path });
}

export async function nextInvestigator(): Promise<string> {
  return invoke<string>("next_investigator", {});
}
