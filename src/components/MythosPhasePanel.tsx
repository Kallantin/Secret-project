import { useGameStore } from "../store/gameStore";

export default function MythosPhasePanel() {
  const game = useGameStore((state) => state.game);
  const lastMythosResult = useGameStore((state) => state.lastMythosResult);
  const busy = useGameStore((state) => state.busy);
  const performMythosPhase = useGameStore((state) => state.performMythosPhase);

  const encounterCount = game?.encounter_deck.length ?? 0;
  const discardCount = game?.encounter_discard.length ?? 0;
  const enemiesCount = game?.enemies_in_play.length ?? 0;

  return (
    <div className="rounded-lg border border-arkham-border bg-arkham-panel p-4">
      <h2 className="mb-3 font-semibold text-arkham-accent">Mythos Phase</h2>

      <div className="mb-3 flex items-center justify-between">
        <button
          type="button"
          onClick={() => void performMythosPhase()}
          disabled={!game || busy || game.phase !== "mythos"}
          className="rounded border border-arkham-border px-3 py-1.5 text-sm text-gray-300 disabled:opacity-40 hover:bg-black/20"
        >
          Draw Encounter Card
        </button>
        <span className="text-xs text-gray-400">
          Deck: {encounterCount} · Discard: {discardCount} · Enemies: {enemiesCount}
        </span>
      </div>

      {!game ? (
        <p className="text-sm text-gray-400">No active game.</p>
      ) : game.phase !== "mythos" ? (
        <p className="text-sm text-gray-400">
          Mythos phase not active (current:{" "}
          {game.phase.charAt(0).toUpperCase() + game.phase.slice(1)}).
        </p>
      ) : lastMythosResult ? (
        <div className="mt-3 pt-3 border-t border-arkham-border/50">
          <p className="text-xs uppercase tracking-wide text-gray-400 mb-1">
            Last Mythos Card
          </p>
          <div className="flex items-start gap-2">
            <div className="flex flex-col">
              <span className="text-sm font-semibold text-gray-200">
                {lastMythosResult.card.name}
              </span>
              <span className="text-[10px] text-gray-500 uppercase tracking-tighter">
                {lastMythosResult.card.card_type}
              </span>
            </div>
            {lastMythosResult.spawned_enemy && (
              <span className="ml-auto rounded bg-red-900 px-2 py-0.5 text-[10px] text-red-200">
                Enemy Spawned
              </span>
            )}
          </div>
          <p className="mt-1 text-sm text-gray-300 whitespace-pre-line">
            {lastMythosResult.card.text}
          </p>
        </div>
      ) : (
        <p className="text-sm text-gray-400 mt-3">
          No mythos card drawn yet this phase.
        </p>
      )}
    </div>
  );
}
