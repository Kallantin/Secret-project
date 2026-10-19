import { create } from "zustand";
import type { ChaosToken, GameState } from "../types";
import {
  addInvestigator as apiAddInvestigator,
  advancePhase as apiAdvancePhase,
  drawChaosToken as apiDrawChaosToken,
  getGameState,
  isTauri,
  loadGame as apiLoadGame,
  newGame as apiNewGame,
  saveGame as apiSaveGame,
} from "../lib/api";

function message(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "string") {
    return error;
  }
  return "Unexpected error";
}

interface InvestigatorSelection {
  code: string;
  name: string;
  willpower: number;
  intellect: number;
  combat: number;
  agility: number;
  max_health: number;
  max_sanity: number;
}

interface GameStore {
  game: GameState | null;
  lastToken: ChaosToken | null;
  busy: boolean;
  error: string | null;
  inTauri: boolean;
  init: () => Promise<void>;
  refresh: () => Promise<void>;
  startNewGame: (
    campaign?: string,
    scenario?: string,
    seed?: number | null,
    investigators?: InvestigatorSelection[],
  ) => Promise<void>;
  addInvestigator: (investigator: InvestigatorSelection) => Promise<void>;
  nextPhase: () => Promise<void>;
  drawToken: () => Promise<void>;
  persist: (path: string) => Promise<void>;
  restore: (path: string) => Promise<void>;
}

export const useGameStore = create<GameStore>((set, get) => ({
  game: null,
  lastToken: null,
  busy: false,
  error: null,
  inTauri: isTauri(),

  init: async () => {
    if (!get().inTauri) {
      return;
    }
    await get().refresh();
  },

  refresh: async () => {
    const game = await getGameState();
    set({ game });
  },

  startNewGame: async (campaign, scenario, seed, investigators) => {
    set({ busy: true, error: null });
    try {
      const game = await apiNewGame(
        campaign ?? "Night of the Zealot",
        scenario ?? "The Gathering",
        seed ?? null,
      );
      set({ game, lastToken: null });

      if (investigators && investigators.length > 0) {
        for (const investigator of investigators) {
          await apiAddInvestigator(investigator);
        }
        await get().refresh();
      }
    } catch (error) {
      set({ error: message(error) });
    } finally {
      set({ busy: false });
    }
  },

  addInvestigator: async (investigator) => {
    set({ busy: true, error: null });
    try {
      await apiAddInvestigator(investigator);
      await get().refresh();
    } catch (error) {
      set({ error: message(error) });
    } finally {
      set({ busy: false });
    }
  },

  nextPhase: async () => {
    set({ busy: true, error: null });
    try {
      await apiAdvancePhase();
      await get().refresh();
    } catch (error) {
      set({ error: message(error) });
    } finally {
      set({ busy: false });
    }
  },

  drawToken: async () => {
    set({ busy: true, error: null });
    try {
      const token = await apiDrawChaosToken();
      set({ lastToken: token });
      await get().refresh();
    } catch (error) {
      set({ error: message(error) });
    } finally {
      set({ busy: false });
    }
  },

  persist: async (path) => {
    set({ busy: true, error: null });
    try {
      await apiSaveGame(path);
    } catch (error) {
      set({ error: message(error) });
    } finally {
      set({ busy: false });
    }
  },

  restore: async (path) => {
    set({ busy: true, error: null });
    try {
      const game = await apiLoadGame(path);
      set({ game, lastToken: null });
    } catch (error) {
      set({ error: message(error) });
    } finally {
      set({ busy: false });
    }
  },
}));
