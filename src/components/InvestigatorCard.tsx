import type { Investigator } from "../types";

function Stat({ label, value }: { label: string; value: number }) {
  return (
    <div className="flex flex-col items-center rounded bg-black/30 px-2 py-1">
      <span className="text-xs uppercase tracking-wide text-gray-400">
        {label}
      </span>
      <span className="text-sm font-semibold">{value}</span>
    </div>
  );
}

export default function InvestigatorCard({
  investigator,
}: {
  investigator: Investigator;
}) {
  return (
    <article className="rounded-lg border border-arkham-border bg-arkham-panel p-4">
      <header className="mb-3 flex items-center justify-between">
        <h2 className="font-semibold">{investigator.name}</h2>
        {investigator.is_defeated && (
          <span className="rounded bg-red-900 px-2 py-0.5 text-xs text-red-200">
            Defeated
          </span>
        )}
      </header>

      <div className="mb-3 grid grid-cols-4 gap-2">
        <Stat label="Will" value={investigator.willpower} />
        <Stat label="Int" value={investigator.intellect} />
        <Stat label="Com" value={investigator.combat} />
        <Stat label="Agi" value={investigator.agility} />
      </div>

      <div className="grid grid-cols-2 gap-2 text-sm">
        <span>
          Health: {investigator.health}/{investigator.max_health}
        </span>
        <span>
          Sanity: {investigator.sanity}/{investigator.max_sanity}
        </span>
        <span>Actions: {investigator.actions_remaining}</span>
        <span>Resources: {investigator.resources}</span>
        <span>Clues: {investigator.clues}</span>
      </div>
    </article>
  );
}
