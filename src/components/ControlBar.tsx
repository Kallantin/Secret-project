import { useGameStore } from "../store/gameStore";
import type { Phase } from "../types";

const PHASE_LABELS: Record<Phase, string> = {
  mythos: "Mythos",
  investigation: "Investigation",
  enemy: "Enemy",
  upkeep: "Upkeep",
};

interface ControlBarProps {
  onNewGame: () => void;
}

export default function ControlBar({ onNewGame }: ControlBarProps) {
  const game = useGameStore((state) => state.game);
  const inTauri = useGameStore((state) => state.inTauri);
  const busy = useGameStore((state) => state.busy);
  const nextPhase = useGameStore((state) => state.nextPhase);
  const drawToken = useGameStore((state) => state.drawToken);

  return (
    <header className="flex items-center justify-between border-b border-arkham-border bg-arkham-panel px-4 py-3">
      <div className="flex items-center gap-3">
        <h1 className="text-lg font-semibold text-arkham-accent">
          Arkham Horror LCG
        </h1>
        {game && (
          <span className="text-sm text-gray-400">
            Round {game.round} · {PHASE_LABELS[game.phase] ?? game.phase}
          </span>
        )}
      </div>

      {game?.game_status !== "ongoing" && (
        <div className="flex items-center justify-center px-4 py-2">
          <span
            className={`px-3 py-1 rounded text-xs font-bold ${
              game?.game_status === "won"
                ? "bg-green-900/20 text-green-400 border border-green-500/50"
                : "bg-red-900/20 text-red-400 border border-red-500/50"
            }`}
          >
            {game?.game_status === "won" ? "VICTORY" : "DEFEAT"}
          </span>
        </div>
      )}

      <div className="flex items-center gap-2">
        <button
          type="button"
          className="rounded bg-arkham-accent px-3 py-1.5 text-sm font-medium text-black disabled:opacity-50"
          disabled={!inTauri || busy}
          onClick={onNewGame}
        >
          New Game
        </button>
        <button
          type="button"
          className="rounded border border-arkham-border px-3 py-1.5 text-sm disabled:opacity-50"
          disabled={!game || busy || game?.game_status !== "ongoing"}
          onClick={() => void nextPhase()}
        >
          Next Phase
        </button>
        <button
          type="button"
          className="rounded border border-arkham-border px-3 py-1.5 text-sm disabled:opacity-50"
          disabled={!game || busy || game?.game_status !== "ongoing"}
          onClick={() => void drawToken()}
        >
          Draw Token
        </button>
      </div>
    </header>
  );
}
