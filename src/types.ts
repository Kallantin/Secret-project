export type Phase = "mythos" | "investigation" | "enemy" | "upkeep";

export type ChaosToken =
  | { kind: "numeric"; value: number }
  | { kind: "skull" }
  | { kind: "cultist" }
  | { kind: "tablet" }
  | { kind: "elder_thing" }
  | { kind: "auto_fail" }
  | { kind: "elder_sign" }
  | { kind: "bless" }
  | { kind: "curse" };

export interface ChaosBag {
  tokens: ChaosToken[];
  seed: number;
  draws: number;
}

export interface SkillTestResult {
  investigator_id: string;
  investigator_name: string;
  skill: string;
  stat_value: number;
  token: ChaosToken;
  token_modifier: number;
  total: number;
}

export interface MythosResult {
  card: Card;
  spawned_enemy: Card | null;
  discarded: boolean;
}

export interface PlayCardResult {
  investigator_id: string;
  card: Card;
  remaining_resources: number;
}

export interface SkillIcons {
  willpower: number;
  intellect: number;
  combat: number;
  agility: number;
  wild: number;
}

export interface Card {
  id: string;
  code: string;
  name: string;
  card_type: string;
  class: string | null;
  cost: number | null;
  xp: number;
  icons: SkillIcons;
  traits: string[];
  text: string;
}

export interface DeckSlot {
  code: string;
  quantity: number;
}

export interface Deck {
  draw_pile: Card[];
  hand: Card[];
  discard_pile: Card[];
  play_area: Card[];
  removed_from_game: Card[];
  decklist: DeckSlot[];
}

export interface Investigator {
  id: string;
  code: string;
  name: string;
  willpower: number;
  intellect: number;
  combat: number;
  agility: number;
  health: number;
  max_health: number;
  sanity: number;
  max_sanity: number;
  resources: number;
  actions_remaining: number;
  clues: number;
  is_defeated: boolean;
  deck: Deck;
}

export interface GameState {
  id: string;
  campaign: string;
  scenario: string;
  round: number;
  phase: Phase;
  active_investigator: number;
  investigators: Investigator[];
  chaos_bag: ChaosBag;
  encounter_deck: Card[];
  encounter_discard: Card[];
  enemies_in_play: Card[];
  log: string[];
}
