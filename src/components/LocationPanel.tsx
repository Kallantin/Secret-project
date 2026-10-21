import { useGameStore } from "../store/gameStore";

export default function LocationPanel() {
  const game = useGameStore((state) => state.game);
  const lastMoveResult = useGameStore((state) => state.lastMoveResult);
  const lastInvestigateResult = useGameStore(
    (state) => state.lastInvestigateResult,
  );
  const busy = useGameStore((state) => state.busy);
  const moveToLocation = useGameStore((state) => state.moveToLocation);
  const investigate = useGameStore((state) => state.investigate);

  const investigator = game
    ? game.investigators[game.active_investigator]
    : undefined;

  const locations = game?.locations_in_play ?? [];
  const currentLocationId = investigator?.location_id ?? null;

  const handleMove = (locationId: string) => {
    if (!investigator) return;
    void moveToLocation(investigator.id, locationId);
  };

  const handleInvestigate = () => {
    if (!investigator) return;
    void investigate(investigator.id);
  };

  if (!game) {
    return (
      <div className="rounded-lg border border-arkham-border bg-arkham-panel p-4">
        <h2 className="mb-3 font-semibold text-arkham-accent">Locations</h2>
        <p className="text-sm text-gray-400">No active game.</p>
      </div>
    );
  }

  return (
    <div className="rounded-lg border border-arkham-border bg-arkham-panel p-4">
      <h2 className="mb-3 font-semibold text-arkham-accent">Locations</h2>

      <div className="mb-4">
        <p className="text-xs uppercase tracking-wide text-gray-400 mb-2">
          Locations in Play ({locations.length})
        </p>
        {locations.length === 0 ? (
          <p className="text-sm text-gray-400">No locations in play.</p>
        ) : (
          <div className="space-y-2">
            {locations.map((location) => {
              const isHere = currentLocationId === location.id;
              return (
                <div
                  key={location.id}
                  className="flex items-center justify-between gap-2 rounded border border-arkham-border/50 bg-black/20 p-2"
                >
                  <div className="flex-shrink-0 w-16 h-10 rounded border border-arkham-border bg-black/40 flex items-center justify-center text-[10px] font-semibold text-gray-400">
                    {location.card.name
                      .split(" ")
                      .map((w) => w[0])
                      .join("")
                      .toUpperCase()}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-1">
                      <span className="text-sm font-semibold text-gray-200 truncate">
                        {location.card.name}
                      </span>
                      {isHere && (
                        <span className="rounded bg-arkham-accent px-1.5 py-0 text-[10px] font-bold text-black">
                          Here
                        </span>
                      )}
                    </div>
                    <div className="text-[10px] text-gray-500 mt-0.5">
                      Clues: {location.clues_remaining}
                    </div>
                    <div className="text-[10px] text-gray-500 mt-0.5">
                      Shroud: {
                        (location.card.raw as { shroud?: number }).shroud ?? 1
                      }
                    </div>
                  </div>
                  <div className="flex gap-1">
                    {!isHere && (
                      <button
                        type="button"
                        onClick={() => void handleMove(location.id)}
                        disabled={busy || !investigator}
                        className="rounded border border-arkham-border px-2 py-0.5 text-[10px] text-gray-300 disabled:opacity-40 hover:bg-black/20"
                      >
                        Move
                      </button>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      <div className="mb-4">
        <p className="text-xs uppercase tracking-wide text-gray-400 mb-2">
          Current Location
        </p>
        {currentLocationId ? (
          <div className="flex items-center gap-2 text-sm">
            <span className="font-semibold text-gray-200">
              {locations.find((l) => l.id === currentLocationId)?.card.name ??
                "Unknown"}
            </span>
            <span className="text-xs text-gray-500">
              Clues remaining:{" "}
              {locations.find((l) => l.id === currentLocationId)?.clues_remaining ??
                0}
            </span>
          </div>
        ) : (
          <p className="text-sm text-gray-400">Not at a location.</p>
        )}
      </div>

      {currentLocationId && (
        <div className="mb-4">
          <button
            type="button"
            onClick={handleInvestigate}
            disabled={
              busy ||
              !investigator ||
              investigator.actions_remaining === 0 ||
              locations.find((l) => l.id === currentLocationId)
                ?.clues_remaining === 0
            }
            className="w-full rounded border border-arkham-border px-3 py-1.5 text-sm text-gray-300 disabled:opacity-40 hover:bg-black/20"
          >
            Investigate ({investigator?.actions_remaining ?? 0} actions)
          </button>
        </div>
      )}

      {lastMoveResult && (
        <div className="mb-3 pt-3 border-t border-arkham-border/50">
          <p className="text-xs uppercase tracking-wide text-gray-400 mb-1">
            Last Move
          </p>
          <div className="text-sm text-green-400">
            {lastMoveResult.location_name}
          </div>
        </div>
      )}

      {lastInvestigateResult && (
        <div className="pt-3 border-t border-arkham-border/50">
          <p className="text-xs uppercase tracking-wide text-gray-400 mb-1">
            Last Investigation
          </p>
          <div className="text-sm text-gray-200">
            {lastInvestigateResult.clues_discovered > 0 ? (
              <>
                Found {lastInvestigateResult.clues_discovered} clue{" "}
                {lastInvestigateResult.clues_discovered === 1
                  ? ""
                  : "s"}. Remaining:{" "}
                {lastInvestigateResult.clues_remaining}
              </>
            ) : (
              <span>No clues found.</span>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
