import type { Investigator } from "../types";
import { INVESTIGATORS } from "../data/investigators";

const CLASS_COLORS: Record<string, string> = {
  guardian: "bg-red-500/20 border-red-500/50 text-red-400",
  seeker: "bg-blue-500/20 border-blue-500/50 text-blue-400",
  mystic: "bg-purple-500/20 border-purple-500/50 text-purple-400",
  rogue: "bg-green-500/20 border-green-500/50 text-green-400",
  survivor: "bg-yellow-500/20 border-yellow-500/50 text-yellow-400",
  neutral: "bg-gray-500/20 border-gray-500/50 text-gray-400",
};

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

function investigatorImage(code: string): string | undefined {
  const imageFile = `${code}.png`;
  return `/assets/cards/${imageFile}`;
}

function getInvestigatorClass(code: string): string | undefined {
  const template = INVESTIGATORS.find((inv) => inv.code === code);
  return template?.class;
}

export default function InvestigatorCard({
  investigator,
}: {
  investigator: Investigator;
}) {
  const investigatorClass = getInvestigatorClass(investigator.code);
  const classColor = CLASS_COLORS[investigatorClass ?? "neutral"];
  const classColorText = classColor?.split(" ").pop() ?? "text-gray-400";
  const imageSrc = investigatorImage(investigator.code);

  return (
    <article className="rounded-lg border border-arkham-border bg-arkham-panel p-4 flex flex-col">
      {/* Investigator card image placeholder */}
      <div className="aspect-[227/325] rounded-t border border-arkham-border bg-black/40 overflow-hidden relative mb-3">
        <img
          src={imageSrc}
          alt={investigator.name}
          className="w-full h-full object-cover"
          onError={(e) => {
            (e.target as HTMLImageElement).style.display = "none";
            (e.target as HTMLImageElement).parentElement!.className =
              "aspect-[227/325] rounded-t border border-arkham-border flex items-center justify-center bg-black/40";
          }}
        />
        <div className="absolute inset-0 flex items-center justify-center bg-black/60 text-white font-bold text-lg">
          {investigator.name.split(" ").map((w) => w[0]).join("")}
        </div>
      </div>

      <header className="mb-3 flex items-center justify-between">
        <h2 className={`font-semibold ${classColorText}`}>
          {investigator.name}
        </h2>
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
