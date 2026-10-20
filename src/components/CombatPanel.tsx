import { useGameStore } from "../store/gameStore";

export default function CombatPanel() {
  const game = useGameStore((state) => state.game);
  const lastEngageResult = useGameStore((state) => state.lastEngageResult);
  const lastFightResult = useGameStore((state) => state.lastFightResult);
  const busy = useGameStore((state) => state.busy);
  const engageEnemy = useGameStore((state) => state.engageEnemy);
  const fightEnemy = useGameStore((state) => state.fightEnemy);

  const investigator = game
    ? game.investigators[game.active_investigator]
    : undefined;

  const enemies = game?.enemies_in_play ?? [];
  const engagedIds = new Set(investigator?.engaged_enemies ?? []);

  const handleEngage = (enemyId: string) => {
    if (!investigator) return;
    void engageEnemy(investigator.id, enemyId);
  };

  const handleFight = (enemyId: string) => {
    if (!investigator) return;
    void fightEnemy(investigator.id, enemyId);
  };

  if (!game) {
    return (
      <div className="rounded-lg border border-arkham-border bg-arkham-panel p-4">
        <h2 className="mb-3 font-semibold text-arkham-accent">Combat</h2>
        <p className="text-sm text-gray-400">No active game.</p>
      </div>
    );
  }

  return (
    <div className="rounded-lg border border-arkham-border bg-arkham-panel p-4">
      <h2 className="mb-3 font-semibold text-arkham-accent">Combat</h2>

      <div className="mb-4">
        <p className="text-xs uppercase tracking-wide text-gray-400 mb-2">
          Enemies in Play ({enemies.length})
        </p>
        {enemies.length === 0 ? (
          <p className="text-sm text-gray-400">No enemies in play.</p>
        ) : (
          <div className="space-y-2">
            {enemies.map((enemy) => {
              const isEngaged = engagedIds.has(enemy.id);
              const maxHealth = enemy.card.raw.health as number;
              return (
                <div
                  key={enemy.id}
                  className="flex items-center justify-between gap-2 rounded border border-arkham-border/50 bg-black/20 p-2"
                >
                  <div className="flex-1">
                    <div className="flex items-center gap-1">
                      <span className="text-sm font-semibold text-gray-200">
                        {enemy.card.name}
                      </span>
                      {isEngaged && (
                        <span className="rounded bg-arkham-accent px-1.5 py-0 text-[10px] font-bold text-black">
                          Engaged
                        </span>
                      )}
                    </div>
                    <div className="text-[10px] text-gray-500 mt-0.5">
                      Health: {enemy.health}/{maxHealth}
                    </div>
                    {enemy.engaged_with && (
                      <div className="text-[10px] text-gray-500 mt-0.5">
                        Engaged with another investigator
                      </div>
                    )}
                  </div>
                  <div className="flex gap-1">
                    {!isEngaged && (
                      <button
                        type="button"
                        onClick={() => void handleEngage(enemy.id)}
                        disabled={busy || !investigator}
                        className="rounded border border-arkham-border px-2 py-0.5 text-[10px] text-gray-300 disabled:opacity-40 hover:bg-black/20"
                      >
                        Engage
                      </button>
                    )}
                    {isEngaged && investigator && (
                      <button
                        type="button"
                        onClick={() => void handleFight(enemy.id)}
                        disabled={
                          busy ||
                          !investigator ||
                          investigator.actions_remaining === 0
                        }
                        className="rounded border border-arkham-border px-2 py-0.5 text-[10px] text-gray-300 disabled:opacity-40 hover:bg-black/20"
                      >
                        Fight
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
          Active Investigator:{" "}
          {investigator ? investigator.name : "—"}
        </p>
        {investigator && (
          <p className="text-xs text-gray-500">
            Engaged enemies:{" "}
            {investigator.engaged_enemies.length > 0
              ? investigator.engaged_enemies
                  .map((id) => {
                    const e = enemies.find((en) => en.id === id);
                    return e ? e.card.name : id;
                  })
                  .join(", ")
              : "None"}
          </p>
        )}
      </div>

      {lastEngageResult && (
        <div className="mb-3 pt-3 border-t border-arkham-border/50">
          <p className="text-xs uppercase tracking-wide text-gray-400 mb-1">
            Last Engage
          </p>
          <div className="text-sm text-green-400">
            {investigator?.name ?? "Investigator"} engaged{" "}
            {lastEngageResult.enemy_name}
          </div>
        </div>
      )}

      {lastFightResult && (
        <div className="pt-3 border-t border-arkham-border/50">
          <p className="text-xs uppercase tracking-wide text-gray-400 mb-1">
            Last Fight
          </p>
          <div className="text-sm text-gray-200">
            {lastFightResult.success ? (
              <>
                Hit for {lastFightResult.damage_dealt} damage. Enemy health:{" "}
                {lastFightResult.enemy_health}
              </>
            ) : (
              <>
                Missed. Took {lastFightResult.damage_taken} damage. ({" "}
                {lastFightResult.skill_test.total})
              </>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
