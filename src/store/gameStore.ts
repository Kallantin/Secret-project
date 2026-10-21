import { create } from "zustand";
import type { CardTemplate } from "../data/cards";
import type {
  AdvanceActResult,
  CampaignProgress,
  ChaosToken,
  DeckSlot,
  EngageResult,
  FightResult,
  GameState,
  InvestigateResult,
  MoveResult,
  MythosResult,
  PlaceDoomResult,
  ScenarioData,
  SkillTestResult,
} from "../types";
import { useMultiplayerStore } from "./multiplayerStore";
import {
  addCampaignLogEntry as apiAddCampaignLogEntry,
  addInvestigator as apiAddInvestigator,
  advanceAct as apiAdvanceAct,
  advancePhase as apiAdvancePhase,
  drawChaosToken as apiDrawChaosToken,
  engage as apiEngage,
  fight as apiFight,
  getGameState,
  investigate as apiInvestigate,
  isTauri,
  loadGame as apiLoadGame,
  moveToLocation as apiMoveToLocation,
  newGame as apiNewGame,
  mythosPhase as apiMythosPhase,
  placeDoom as apiPlaceDoom,
  playCard as apiPlayCard,
  resetTurn as apiResetTurn,
  saveGame as apiSaveGame,
  skillTest as apiSkillTest,
  spendAction as apiSpendAction,
  startNextScenario as apiStartNextScenario,
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
    scenarioData?: ScenarioData,
    campaignProgress?: CampaignProgress | null,
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
  engageEnemy: (investigatorId: string, enemyInstanceId: string) => Promise<void>;
  fightEnemy: (investigatorId: string, enemyInstanceId: string) => Promise<void>;
  lastEngageResult: EngageResult | null;
  lastFightResult: FightResult | null;
  lastInvestigateResult: InvestigateResult | null;
  lastMoveResult: MoveResult | null;
  lastPlaceDoomResult: PlaceDoomResult | null;
  lastAdvanceActResult: AdvanceActResult | null;
  moveToLocation: (
    investigatorId: string,
    locationInstanceId: string,
  ) => Promise<void>;
  investigate: (investigatorId: string) => Promise<void>;
  placeDoom: (amount: number) => Promise<void>;
  advanceAct: (contributions: Array<{ investigatorId: string; cluesSpent: number }>) => Promise<void>;
  spendXp: (investigatorId: string, cardCode: string, xpCost: number) => Promise<void>;
  startNextScenario: (
    scenario: string,
    seed: number | null,
    scenarioData: ScenarioData | undefined,
  ) => Promise<void>;
  addCampaignLogEntry: (result: string, notes: string) => Promise<void>;
  applyGameState: (gameState: unknown) => void;
  broadcastGameState: () => void;
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
  lastEngageResult: null,
  lastFightResult: null,
  lastInvestigateResult: null,
  lastMoveResult: null,
  lastPlaceDoomResult: null,
  lastAdvanceActResult: null,

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

  startNewGame: async (
    campaign?: string,
    scenario?: string,
    seed?: number | null,
    investigators?: InvestigatorSelection[],
    encounterCards?: CardTemplate[],
    scenarioData?: ScenarioData,
    campaignProgress?: CampaignProgress | null,
  ) => {
    set({ busy: true, error: null });
    try {
      const game = await apiNewGame(
        campaign ?? "Night of the Zealot",
        scenario ?? "The Gathering",
        seed ?? null,
        encounterCards ?? [],
        scenarioData,
        campaignProgress ?? null,
      );
      set({ game, lastToken: null });

      if (investigators && investigators.length > 0) {
        for (const investigator of investigators) {
          await apiAddInvestigator(investigator);
        }
        await get().refresh();
      }
      useMultiplayerStore.getState().broadcastAction('new_game', {
        campaign: campaign ?? "Night of the Zealot",
        scenario: scenario ?? "The Gathering",
        seed,
      });
    } catch (error) {
      set({ error: message(error) });
    } finally {
      set({ busy: false });
    }
  },

  startNextScenario: async (scenario, seed, scenarioData) => {
    set({ busy: true, error: null });
    try {
      const game = await apiStartNextScenario(scenario, seed ?? null, scenarioData);
      set({ game, lastToken: null });
      await get().refresh();
    } catch (error) {
      set({ error: message(error) });
    } finally {
      set({ busy: false });
    }
  },

  addCampaignLogEntry: async (result, notes) => {
    set({ busy: true, error: null });
    try {
      const game = await apiAddCampaignLogEntry(result, notes);
      set({ game, lastToken: null });
      await get().refresh();
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
      useMultiplayerStore.getState().broadcastAction('add_investigator', {
        code: investigator.code,
        name: investigator.name,
        willpower: investigator.willpower,
        intellect: investigator.intellect,
        combat: investigator.combat,
        agility: investigator.agility,
        max_health: investigator.max_health,
        max_sanity: investigator.max_sanity,
      });
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
      useMultiplayerStore.getState().broadcastAction('advance_phase', {});
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
      useMultiplayerStore.getState().broadcastAction('draw_chaos_token', {});
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
      useMultiplayerStore.getState().broadcastAction('skill_test', {
        investigatorId,
        skill,
      });
    } catch (error) {
      set({ error: message(error), busy: false });
    }
  },

  spendInvestigatorAction: async (investigatorId) => {
    set({ busy: true, error: null });
    try {
      await apiSpendAction(investigatorId);
      await get().refresh();
      useMultiplayerStore.getState().broadcastAction('spend_action', {
        investigatorId,
      });
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
      useMultiplayerStore.getState().broadcastAction('reset_turn', {});
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
      useMultiplayerStore.getState().broadcastAction('mythos_phase', {});
    } catch (error) {
      set({ error: message(error), busy: false });
    }
  },

  fightEnemy: async (investigatorId, enemyInstanceId) => {
    set({ busy: true, error: null });
    try {
      const result = await apiFight(investigatorId, enemyInstanceId);
      set({ lastFightResult: result, busy: false });
      await get().refresh();
      useMultiplayerStore.getState().broadcastAction('fight', {
        investigatorId,
        enemyInstanceId,
      });
    } catch (error) {
      set({ error: message(error) });
    } finally {
      set({ busy: false });
    }
  },

  engageEnemy: async (investigatorId: string, enemyInstanceId: string) => {
    set({ busy: true, error: null });
    try {
      const result = await apiEngage(investigatorId, enemyInstanceId);
      set({ lastEngageResult: result, busy: false });
      await get().refresh();
      useMultiplayerStore.getState().broadcastAction('engage', {
        investigatorId,
        enemyInstanceId,
      });
    } catch (error) {
      set({ error: message(error) });
    } finally {
      set({ busy: false });
    }
  },

  spendXp: async (investigatorId: string, cardCode: string, xpCost: number) => {
    set({ busy: true, error: null });
    try {
      const { game } = get();
      const investigator = game?.investigators.find((inv) => inv.id === investigatorId);
      if (!investigator) {
        set({ error: "Investigator not found" });
        return;
      }
      if (investigator.xp < xpCost) {
        set({ error: "Not enough XP" });
        return;
      }
      investigator.xp -= xpCost;
      investigator.deck.decklist.push({ code: cardCode, quantity: 1 });
      set({ game });
      useMultiplayerStore.getState().broadcastAction('spend_xp', {
        investigatorId,
        cardCode,
        xpCost,
      });
    } catch (error) {
      set({ error: message(error) });
    } finally {
      set({ busy: false });
    }
  },

  applyGameState: (gameState: unknown) => {
    set({ game: gameState as GameState });
  },

  broadcastGameState: async () => {
    const { game } = get();
    if (game) {
      useMultiplayerStore.getState().broadcastAction('game_state_sync', game);
    }
  },

  playInvestigatorCard: async (investigatorId: string, cardId: string) => {
    set({ busy: true, error: null });
    try {
      await apiPlayCard(investigatorId, cardId);
      await get().refresh();
      useMultiplayerStore.getState().broadcastAction('play_card', {
        investigatorId,
        cardId,
      });
    } catch (error) {
      set({ error: message(error) });
    } finally {
      set({ busy: false });
    }
  },

  moveToLocation: async (investigatorId: string, locationInstanceId: string) => {
    set({ busy: true, error: null });
    try {
      const result = await apiMoveToLocation(investigatorId, locationInstanceId);
      set({ lastMoveResult: result, busy: false });
      await get().refresh();
      useMultiplayerStore.getState().broadcastAction('move_to_location', {
        investigatorId,
        locationInstanceId,
      });
    } catch (error) {
      set({ error: message(error) });
    } finally {
      set({ busy: false });
    }
  },

  investigate: async (investigatorId: string) => {
    set({ busy: true, error: null });
    try {
      const result = await apiInvestigate(investigatorId);
      set({ lastInvestigateResult: result, busy: false });
      await get().refresh();
      useMultiplayerStore.getState().broadcastAction('investigate', {
        investigatorId,
      });
    } catch (error) {
      set({ error: message(error) });
    } finally {
      set({ busy: false });
    }
  },

  placeDoom: async (amount: number) => {
    set({ busy: true, error: null });
    try {
      const result = await apiPlaceDoom(amount);
      set({ lastPlaceDoomResult: result, busy: false });
      await get().refresh();
      useMultiplayerStore.getState().broadcastAction('place_doom', { amount });
    } catch (error) {
      set({ error: message(error) });
    } finally {
      set({ busy: false });
    }
  },

  advanceAct: async (
    contributions: Array<{ investigatorId: string; cluesSpent: number }>,
  ) => {
    set({ busy: true, error: null });
    try {
      const result = await apiAdvanceAct(contributions);
      set({ lastAdvanceActResult: result, busy: false });
      await get().refresh();
      useMultiplayerStore.getState().broadcastAction('advance_act', {
        contributions,
      });
    } catch (error) {
      set({ error: message(error) });
    } finally {
      set({ busy: false });
    }
  },
}));

useMultiplayerStore.getState().setRemoteActionHandler((action) => {
  const state = useGameStore.getState();
  if (!state.game) return;

  switch (action.type) {
    case 'skill_test': {
      const { investigatorId, skill } = action.payload as {
        investigatorId: string;
        skill: string;
      };
      void state.performSkillTest(investigatorId, skill);
      break;
    }
    case 'spend_action': {
      const { investigatorId } = action.payload as {
        investigatorId: string;
      };
      void state.spendInvestigatorAction(investigatorId);
      break;
    }
    case 'play_card': {
      const { investigatorId, cardId } = action.payload as {
        investigatorId: string;
        cardId: string;
      };
      void state.playInvestigatorCard(investigatorId, cardId);
      break;
    }
    case 'place_doom': {
      const { amount } = action.payload as { amount: number };
      void state.placeDoom(amount);
      break;
    }
    case 'advance_act': {
      const { contributions } = action.payload as {
        contributions: Array<{ investigatorId: string; cluesSpent: number }>;
      };
      void state.advanceAct(contributions);
      break;
    }
    case 'move_to_location': {
      const { investigatorId, locationInstanceId } = action.payload as {
        investigatorId: string;
        locationInstanceId: string;
      };
      void state.moveToLocation(investigatorId, locationInstanceId);
      break;
    }
    case 'investigate': {
      const { investigatorId } = action.payload as {
        investigatorId: string;
      };
      void state.investigate(investigatorId);
      break;
    }
    case 'engage': {
      const { investigatorId, enemyInstanceId } = action.payload as {
        investigatorId: string;
        enemyInstanceId: string;
      };
      void state.engageEnemy(investigatorId, enemyInstanceId);
      break;
    }
    case 'fight': {
      const { investigatorId, enemyInstanceId } = action.payload as {
        investigatorId: string;
        enemyInstanceId: string;
      };
      void state.fightEnemy(investigatorId, enemyInstanceId);
      break;
    }
    case 'spend_xp': {
      const { investigatorId, cardCode, xpCost } = action.payload as {
        investigatorId: string;
        cardCode: string;
        xpCost: number;
      };
      void state.spendXp(investigatorId, cardCode, xpCost);
      break;
    }
    case 'start_next_scenario': {
      const { scenario, seed, scenario_data } = action.payload as {
        scenario: string;
        seed: number | null;
        scenario_data: ScenarioData;
      };
      void state.startNextScenario(scenario, seed, scenario_data);
      break;
    }
    case 'add_campaign_log_entry': {
      const { result, notes } = action.payload as {
        result: string;
        notes: string;
      };
      void state.addCampaignLogEntry(result, notes);
      break;
    }
    case 'advance_phase':
      void state.nextPhase();
      break;
    case 'reset_turn':
      void state.resetInvestigatorTurn();
      break;
    case 'draw_chaos_token':
      void state.drawToken();
      break;
    case 'game_state_sync':
      void state.applyGameState(action.payload);
      break;
    case 'request_state_sync':
      if (state.game) {
        void state.broadcastGameState();
      }
      break;
    default:
      break;
  }
});
