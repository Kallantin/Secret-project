import { create } from "zustand";
import type { CardTemplate } from "../data/cards";
import type {
  ChaosToken,
  DeckSlot,
  GameState,
  MythosResult,
  SkillTestResult,
} from "../types";
import {
  addInvestigator as apiAddInvestigator,
  advancePhase as apiAdvancePhase,
  drawChaosToken as apiDrawChaosToken,
  getGameState,
  isTauri,
  loadGame as apiLoadGame,
  newGame as apiNewGame,
  mythosPhase as apiMythosPhase,
  playCard as apiPlayCard,
  resetTurn as apiResetTurn,
  saveGame as apiSaveGame,
  skillTest as apiSkillTest,
  spendAction as apiSpendAction,
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
  deckBuilderInvestigator: string | null;
  deckBuilderCards: DeckSlot[];
  init: () => Promise<void>;
  refresh: () => Promise<void>;
  startNewGame: (
    campaign?: string,
    scenario?: string,
    seed?: number | null,
    investigators?: InvestigatorSelection[],
    encounterCards?: CardTemplate[],
  ) => Promise<void>;
  addInvestigator: (investigator: InvestigatorSelection) => Promise<void>;
  nextPhase: () => Promise<void>;
  drawToken: () => Promise<void>;
  persist: (path: string) => Promise<void>;
  restore: (path: string) => Promise<void>;
  setDeckBuilderInvestigator: (code: string | null) => Promise<void>;
  addDeckCard: (code: string) => Promise<void>;
  removeDeckCard: (code: string) => Promise<void>;
  setDeckCardQuantity: (code: string, quantity: number) => Promise<void>;
  clearDeckBuilder: () => Promise<void>;
  saveDeckToInvestigator: () => Promise<void>;
  performSkillTest: (investigatorId: string, skill: string) => Promise<void>;
  spendInvestigatorAction: (investigatorId: string) => Promise<void>;
  resetInvestigatorTurn: () => Promise<void>;
  lastSkillTest: SkillTestResult | null;
  performMythosPhase: () => Promise<void>;
  playInvestigatorCard: (investigatorId: string, cardId: string) => Promise<void>;
  lastMythosResult: MythosResult | null;
}

export const useGameStore = create<GameStore>((set, get) => ({
  game: null,
  lastToken: null,
  busy: false,
  error: null,
  inTauri: isTauri(),
  deckBuilderInvestigator: null,
  deckBuilderCards: [],
  lastSkillTest: null,
  lastMythosResult: null,

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

  startNewGame: async (campaign, scenario, seed, investigators, encounterCards) => {
    set({ busy: true, error: null });
    try {
      const game = await apiNewGame(
        campaign ?? "Night of the Zealot",
        scenario ?? "The Gathering",
        seed ?? null,
        encounterCards ?? [],
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

  setDeckBuilderInvestigator: async (code) => {
    set({ deckBuilderInvestigator: code, deckBuilderCards: [] });
  },

  addDeckCard: async (code) => {
    const { deckBuilderCards } = get();
    const existing = deckBuilderCards.find((slot) => slot.code === code);
    if (existing && existing.quantity >= 2) return;
    set({
      deckBuilderCards: existing
        ? deckBuilderCards.map((slot) =>
            slot.code === code ? { ...slot, quantity: slot.quantity + 1 } : slot
          )
        : [...deckBuilderCards, { code, quantity: 1 }],
    });
  },

  removeDeckCard: async (code) => {
    set({
      deckBuilderCards: get().deckBuilderCards
        .map((slot) =>
          slot.code === code ? { ...slot, quantity: Math.max(0, slot.quantity - 1) } : slot
        )
        .filter((slot) => slot.quantity > 0),
    });
  },

  setDeckCardQuantity: async (code, quantity) => {
    const clamped = Math.min(2, Math.max(0, quantity));
    set({
      deckBuilderCards: get().deckBuilderCards
        .map((slot) => (slot.code === code ? { ...slot, quantity: clamped } : slot))
        .filter((slot) => slot.quantity > 0),
    });
  },

  clearDeckBuilder: async () => {
    set({ deckBuilderInvestigator: null, deckBuilderCards: [] });
  },

  saveDeckToInvestigator: async () => {
    const { game, deckBuilderInvestigator, deckBuilderCards } = get();
    if (!game || !deckBuilderInvestigator) return;
    set({ busy: true, error: null });
    try {
      const updated = {
        ...game,
        investigators: game.investigators.map((inv) =>
          inv.code === deckBuilderInvestigator
            ? { ...inv, deck: { ...inv.deck, decklist: [...deckBuilderCards] } }
            : inv
        ),
      };
      set({ game: updated });
    } catch (error) {
      set({ error: message(error) });
    } finally {
      set({ busy: false });
    }
  },

  performSkillTest: async (investigatorId, skill) => {
    set({ busy: true, error: null });
    try {
      const result = await apiSkillTest(investigatorId, skill);
      set({ lastSkillTest: result, busy: false });
      await get().refresh();
    } catch (error) {
      set({ error: message(error), busy: false });
    }
  },

  spendInvestigatorAction: async (investigatorId) => {
    set({ busy: true, error: null });
    try {
      await apiSpendAction(investigatorId);
      await get().refresh();
    } catch (error) {
      set({ error: message(error) });
    } finally {
      set({ busy: false });
    }
  },

  resetInvestigatorTurn: async () => {
    set({ busy: true, error: null });
    try {
      await apiResetTurn();
      await get().refresh();
    } catch (error) {
      set({ error: message(error) });
    } finally {
      set({ busy: false });
    }
  },

  performMythosPhase: async () => {
    set({ busy: true, error: null });
    try {
      const result = await apiMythosPhase();
      set({ lastMythosResult: result, busy: false });
      await get().refresh();
    } catch (error) {
      set({ error: message(error), busy: false });
    }
  },

  playInvestigatorCard: async (investigatorId, cardId) => {
    set({ busy: true, error: null });
    try {
      await apiPlayCard(investigatorId, cardId);
      await get().refresh();
    } catch (error) {
      set({ error: message(error) });
    } finally {
      set({ busy: false });
    }
  },
}));
