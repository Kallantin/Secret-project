import index from "../../data/cards/index.json";
import encounter from "../../data/cards/encounter.json";
import player from "../../data/cards/player.json";

export interface SkillIconsTemplate {
  willpower: number;
  intellect: number;
  combat: number;
  agility: number;
  wild: number;
}

export interface CardTemplate {
  code: string;
  name: string;
  card_type: string;
  class: string;
  cost: number | null;
  xp: number;
  icons: SkillIconsTemplate;
  traits: string[];
  text: string;
  raw: Record<string, unknown>;
}

const byType: Record<string, CardTemplate[]> = {
  encounter,
  player,
};

export const ENCOUNTER_CARDS: CardTemplate[] = byType[index.encounter] ?? [];
export const PLAYER_CARDS: CardTemplate[] = byType[index.player] ?? [];
export const ALL_CARDS: CardTemplate[] = [...ENCOUNTER_CARDS, ...PLAYER_CARDS];
