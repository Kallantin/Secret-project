import index from "../../data/tokens/index.json";
import chaos from "../../data/tokens/chaos.json";
import game from "../../data/tokens/game.json";

export interface TokenTemplate {
  code: string;
  name: string;
  token_type: string;
  kind: string;
  value: number;
  modifier?: number;
  is_symbol?: boolean;
  image: string | null;
}

const byType: Record<string, TokenTemplate[]> = {
  chaos,
  game,
};

export const CHAOS_TOKENS: TokenTemplate[] = byType[index.chaos] ?? [];
export const GAME_TOKENS: TokenTemplate[] = byType[index.game] ?? [];
export const ALL_TOKENS: TokenTemplate[] = [...CHAOS_TOKENS, ...GAME_TOKENS];
