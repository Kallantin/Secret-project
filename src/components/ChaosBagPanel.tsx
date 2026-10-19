import { useGameStore } from "../store/gameStore";
import type { ChaosToken } from "../types";

function tokenLabel(token: ChaosToken): string {
  switch (token.kind) {
    case "numeric":
      return token.value >= 0 ? `+${token.value}` : `${token.value}`;
    case "skull":
      return "Skull";
    case "cultist":
      return "Cultist";
    case "tablet":
      return "Tablet";
    case "elder_thing":
      return "Elder Thing";
    case "auto_fail":
      return "Auto-Fail";
    case "elder_sign":
      return "Elder Sign";
    case "bless":
      return "Bless";
    case "curse":
      return "Curse";
  }
}

export default function ChaosBagPanel() {
  const game = useGameStore((state) => state.game);
  const lastToken = useGameStore((state) => state.lastToken);

  return (
    <div className="rounded-lg border border-arkham-border bg-arkham-panel p-4">
      <h2 className="mb-3 font-semibold text-arkham-accent">Chaos Bag</h2>
      {game ? (
        <div className="flex flex-col gap-2 text-sm">
          <span>Tokens: {game.chaos_bag.tokens.length}</span>
          <span>Draws: {game.chaos_bag.draws}</span>
          <span>Last token: {lastToken ? tokenLabel(lastToken) : "—"}</span>
        </div>
      ) : (
        <p className="text-sm text-gray-400">No active chaos bag.</p>
      )}
    </div>
  );
}
