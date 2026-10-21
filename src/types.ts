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

export interface EnemyInstance {
  id: string;
  card: Card;
  health: number;
  horror: number;
  engaged_with: string | null;
  /// Location instance ID where an unengaged enemy sits. `null` when engaged.
  location_id: string | null;
  /// Whether the enemy is exhausted (cannot engage, attack, or hunt).
  exhausted: boolean;
}

export interface EngageResult {
  investigator_id: string;
  enemy_instance_id: string;
  enemy_name: string;
}

export interface FightResult {
  investigator_id: string;
  enemy_instance_id: string;
  success: boolean;
  enemy_health: number;
  damage_dealt: number;
  damage_taken: number;
  horror_taken: number;
  skill_test: SkillTestResult;
}

export interface LocationInstance {
  id: string;
  card: Card;
  clues_remaining: number;
}

export interface MoveResult {
  investigator_id: string;
  location_instance_id: string;
  location_name: string;
}

export interface InvestigateResult {
  investigator_id: string;
  location_instance_id: string;
  shroud: number;
  test_total: number;
  successes: number;
  clues_discovered: number;
  clues_remaining: number;
}

export type GameStatus = "ongoing" | "won" | "lost";

export type CampaignResultType = "won" | "lost";

export interface CampaignLogEntry {
  scenario_code: string;
  scenario_name: string;
  result: CampaignResultType;
  investigator_names: string[];
  notes: string;
}

export interface BoonOrBlemish {
  name: string;
  description: string;
  is_boon: boolean;
}

export interface CampaignProgress {
  completed_scenarios: string[];
  campaign_log: CampaignLogEntry[];
  boons_and_blemishes: BoonOrBlemish[];
}

export interface ActInstance {
  id: string;
  card: Card;
  progress: number;
}

export interface AgendaInstance {
  id: string;
  card: Card;
  doom: number;
  doom_threshold: number;
}

export interface PlaceDoomResult {
  agenda_index: number;
  doom: number;
  advanced: boolean;
  next_agenda: string | null;
  game_status: GameStatus;
}

export interface AdvanceActResult {
  act_index: number;
  progress: number;
  advanced: boolean;
  next_act: string | null;
  game_status: GameStatus;
}

export interface ScenarioActData {
  code: string;
  name: string;
  text: string;
  clue_cost: number;
  raw: Record<string, unknown>;
}

export interface ScenarioAgendaData {
  code: string;
  name: string;
  text: string;
  doom_threshold: number;
  raw: Record<string, unknown>;
}

export interface ScenarioLocationData {
  code: string;
  name: string;
  text: string;
  clues: number;
  shroud: number;
  raw: Record<string, unknown>;
}

export interface ScenarioData {
  campaign: string;
  code: string;
  name: string;
  order: number;
  chaos_bag: string;
  acts: ScenarioActData[];
  agendas: ScenarioAgendaData[];
  locations: ScenarioLocationData[];
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
  raw: Record<string, unknown>;
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
  /// Experience points earned across campaign play.
  xp: number;
  /// Permanent physical trauma suffered across the campaign.
  physical_trauma: number;
  /// Permanent mental trauma suffered across the campaign.
  mental_trauma: number;
  /// Whether this investigator has been killed and cannot be used for the remainder of the campaign.
  is_killed: boolean;
  /// Whether this investigator has been driven insane and cannot be used for the remainder of the campaign.
  is_driven_insane: boolean;
  /// Enemy instance IDs currently engaged with this investigator.
  engaged_enemies: string[];
  /// Location instance ID the investigator is currently at, if any.
  location_id: string | null;
  /// Investigator mini token ID used to track this investigator's position on the board.
  investigator_token_id: string | null;
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
  enemies_in_play: EnemyInstance[];
  locations_in_play: LocationInstance[];
  current_act_index: number;
  current_agenda_index: number;
  acts: ActInstance[];
  agendas: AgendaInstance[];
  /// Cards in the victory display, worth experience at scenario end.
  victory_display: Card[];
  game_status: GameStatus;
  /// Campaign-wide progress carried between scenarios.
  campaign_progress: CampaignProgress;
  /// In-game log messages for the current scenario.
  log_messages: string[];
}
