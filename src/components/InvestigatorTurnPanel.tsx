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

const SKILLS: readonly [string, string][] = [
  ["willpower", "Willpower"],
  ["intellect", "Intellect"],
  ["combat", "Combat"],
  ["agility", "Agility"],
];

export default function InvestigatorTurnPanel() {
  const game = useGameStore((state) => state.game);
  const lastSkillTest = useGameStore((state) => state.lastSkillTest);
  const busy = useGameStore((state) => state.busy);
  const performSkillTest = useGameStore((state) => state.performSkillTest);
  const spendInvestigatorAction = useGameStore(
    (state) => state.spendInvestigatorAction,
  );
  const resetInvestigatorTurn = useGameStore(
    (state) => state.resetInvestigatorTurn,
  );

  const investigator = game
    ? game.investigators[game.active_investigator]
    : undefined;

  const handleSkillTest = (skill: string) => {
    if (!investigator) return;
    void performSkillTest(investigator.id, skill);
  };

  const handleSpendAction = () => {
    if (!investigator) return;
    void spendInvestigatorAction(investigator.id);
  };

  if (!game || !investigator) {
    return (
      <div className="rounded-lg border border-arkham-border bg-arkham-panel p-4">
        <h2 className="mb-3 font-semibold text-arkham-accent">
          Investigator Turn
        </h2>
        <p className="text-sm text-gray-400">
          No active investigator. Start a new game to begin.
        </p>
      </div>
    );
  }

  return (
    <div className="rounded-lg border border-arkham-border bg-arkham-panel p-4">
      <h2 className="mb-3 font-semibold text-arkham-accent">
        Investigator Turn
      </h2>
      <div className="mb-3">
        <h3 className="font-semibold text-gray-200">{investigator.name}</h3>
        <p className="text-xs text-gray-400">
          Round {game.round} · Phase:{" "}
          {game.phase.charAt(0).toUpperCase() + game.phase.slice(1)}
        </p>
      </div>

      <div className="mb-4 grid grid-cols-4 gap-2">
        <div className="flex flex-col items-center rounded bg-black/30 px-2 py-1">
          <span className="text-[10px] uppercase tracking-wide text-gray-400">
            Will
          </span>
          <span className="text-sm font-semibold">{investigator.willpower}</span>
        </div>
        <div className="flex flex-col items-center rounded bg-black/30 px-2 py-1">
          <span className="text-[10px] uppercase tracking-wide text-gray-400">
            Int
          </span>
          <span className="text-sm font-semibold">{investigator.intellect}</span>
        </div>
        <div className="flex flex-col items-center rounded bg-black/30 px-2 py-1">
          <span className="text-[10px] uppercase tracking-wide text-gray-400">
            Com
          </span>
          <span className="text-sm font-semibold">{investigator.combat}</span>
        </div>
        <div className="flex flex-col items-center rounded bg-black/30 px-2 py-1">
          <span className="text-[10px] uppercase tracking-wide text-gray-400">
            Agi
          </span>
          <span className="text-sm font-semibold">{investigator.agility}</span>
        </div>
      </div>

      <div className="mb-4 grid grid-cols-4 gap-2 text-sm">
        <span>
          Health: {investigator.health}/{investigator.max_health}
        </span>
        <span>
          Sanity: {investigator.sanity}/{investigator.max_sanity}
        </span>
        <span>Resources: {investigator.resources}</span>
        <span>Actions: {investigator.actions_remaining}</span>
      </div>

      <div className="mb-3 flex flex-wrap gap-2">
        <button
          type="button"
          onClick={handleSpendAction}
          disabled={!game || busy || investigator.actions_remaining === 0}
          className="rounded border border-arkham-border px-3 py-1.5 text-sm text-gray-300 disabled:opacity-40 hover:bg-black/20"
        >
          Spend Action
        </button>
        <button
          type="button"
          onClick={resetInvestigatorTurn}
          disabled={busy}
          className="rounded border border-arkham-border px-3 py-1.5 text-sm text-gray-300 disabled:opacity-40 hover:bg-black/20"
        >
          Reset Turn
        </button>
      </div>

      <div className="mb-3">
        <p className="text-xs uppercase tracking-wide text-gray-400 mb-2">
          Skill Test
        </p>
        <div className="flex flex-wrap gap-2">
          {SKILLS.map(([skill, label]) => (
            <button
              type="button"
              key={skill}
              onClick={() => handleSkillTest(skill)}
              disabled={busy || investigator.actions_remaining === 0}
              className="rounded border border-arkham-border px-3 py-1.5 text-sm text-gray-300 disabled:opacity-40 hover:bg-black/20"
            >
              {label}
            </button>
          ))}
        </div>
      </div>

      {lastSkillTest && (
        <div className="mt-4 pt-4 border-t border-arkham-border/50">
          <p className="text-xs uppercase tracking-wide text-gray-400 mb-1">
            Last Test
          </p>
          <div className="text-sm text-gray-200">
            {lastSkillTest.investigator_name} tested {lastSkillTest.skill}{" "}
            (stat {lastSkillTest.stat_value}) +{" "}
            {lastSkillTest.token_modifier} ={" "}
            <span className="font-semibold">{lastSkillTest.total}</span>{" "}
            ({tokenLabel(lastSkillTest.token)})
          </div>
        </div>
      )}
    </div>
  );
}
