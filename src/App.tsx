import { useEffect, useState } from "react";
import CardLibrary from "./components/CardLibrary";
import ChaosBagPanel from "./components/ChaosBagPanel";
import ControlBar from "./components/ControlBar";
import DeckBuilder from "./components/DeckBuilder";
import GameLog from "./components/GameLog";
import InvestigatorCard from "./components/InvestigatorCard";
import InvestigatorPicker from "./components/InvestigatorPicker";
import InvestigatorTurnPanel from "./components/InvestigatorTurnPanel";
import MythosPhasePanel from "./components/MythosPhasePanel";
import CombatPanel from "./components/CombatPanel";
import LocationPanel from "./components/LocationPanel";
import AgendaPanel from "./components/AgendaPanel";
import { type InvestigatorTemplate } from "./data/investigators";
import { ENCOUNTER_CARDS } from "./data/cards";
import { useGameStore } from "./store/gameStore";
import type { Investigator } from "./types";

export default function App() {
  const game = useGameStore((state) => state.game);
  const inTauri = useGameStore((state) => state.inTauri);
  const error = useGameStore((state) => state.error);
  const init = useGameStore((state) => state.init);
  const startNewGame = useGameStore((state) => state.startNewGame);
  const [pickerOpen, setPickerOpen] = useState(false);

  useEffect(() => {
    void init();
  }, [init]);

  const handleNewGame = () => {
    setPickerOpen(true);
  };

  const handlePickerConfirm = (investigators: InvestigatorTemplate[]) => {
    const selections = investigators.map((inv) => ({
      code: inv.code,
      name: inv.name,
      willpower: inv.willpower,
      intellect: inv.intellect,
      combat: inv.combat,
      agility: inv.agility,
      max_health: inv.max_health,
      max_sanity: inv.max_sanity,
    }));
    void startNewGame(undefined, undefined, undefined, selections, ENCOUNTER_CARDS);
  };

  return (
    <div className="flex h-full flex-col">
      <ControlBar onNewGame={handleNewGame} />

      {!inTauri && (
        <div className="border-b border-arkham-border bg-arkham-panel px-4 py-2 text-sm text-arkham-accent">
          Running outside Tauri. Launch the desktop app with{" "}
          <code className="font-mono">corepack pnpm tauri dev</code> to use the
          engine.
        </div>
      )}

      {error && (
        <div className="border-b border-red-900 bg-red-950 px-4 py-2 text-sm text-red-200">
          {error}
        </div>
      )}

      <main className="grid flex-1 grid-cols-[1fr_20rem] gap-4 overflow-hidden p-4">
        <section className="flex flex-col gap-4 overflow-auto">
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
            {game?.investigators.map((investigator) => (
              <InvestigatorCard
                key={investigator.id}
                investigator={investigator as Investigator}
              />
            ))}
            {!game || game.investigators.length === 0 ? (
              <p className="text-sm text-gray-400">
                No active game. Start a new campaign from the toolbar.
              </p>
            ) : null}
          </div>
          <GameLog entries={game?.log ?? []} />
        </section>

        <aside className="flex flex-col gap-4 overflow-auto">
          <ChaosBagPanel />
          <MythosPhasePanel />
          <AgendaPanel />
          <LocationPanel />
          <CombatPanel />
          <InvestigatorTurnPanel />
          <CardLibrary />
          <DeckBuilder />
        </aside>
      </main>

      <InvestigatorPicker
        open={pickerOpen}
        onOpenChange={setPickerOpen}
        onConfirm={handlePickerConfirm}
      />
    </div>
  );
}
