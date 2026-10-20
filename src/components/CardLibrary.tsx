import { useMemo, useState } from "react";
import type { CardTemplate } from "../data/cards";
import { ENCOUNTER_CARDS, PLAYER_CARDS } from "../data/cards";

const CARDS: CardTemplate[] = [...PLAYER_CARDS, ...ENCOUNTER_CARDS];

const SKILLS: readonly [keyof CardTemplate["icons"], string][] = [
  ["willpower", "W"],
  ["intellect", "I"],
  ["combat", "C"],
  ["agility", "A"],
  ["wild", "*"],
];

export default function CardLibrary() {
  const [query, setQuery] = useState("");

  const groups = useMemo(() => {
    const q = query.toLowerCase();
    const map = new Map<string, CardTemplate[]>();
    for (const card of CARDS) {
      const matches =
        card.name.toLowerCase().includes(q) ||
        card.card_type.toLowerCase().includes(q) ||
        card.class.toLowerCase().includes(q) ||
        card.traits.some((t) => t.toLowerCase().includes(q)) ||
        card.text.toLowerCase().includes(q);
      if (!matches) continue;
      const bucket = map.get(card.card_type) ?? [];
      bucket.push(card);
      map.set(card.card_type, bucket);
    }
    return [...map.entries()].sort(([a], [b]) => a.localeCompare(b));
  }, [query]);

  return (
    <section className="flex flex-col gap-4">
      <header className="flex items-center justify-between border-b border-arkham-border pb-2">
        <h2 className="font-semibold">Card Library</h2>
        <span className="text-xs text-gray-400">{CARDS.length} cards</span>
      </header>
      <div className="relative">
        <input
          className="w-full rounded bg-arkham-panel border border-arkham-border px-3 py-1.5 text-sm focus:outline-none focus:ring-1 focus:ring-arkham-accent"
          placeholder="Search cards..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
      </div>
      {groups.length === 0 ? (
        <p className="text-sm text-gray-400">No matching cards.</p>
      ) : (
        <ul className="divide-y divide-arkham-border/50">
          {groups.map(([type, cards]) => (
            <li key={type} className="p-3">
              <h3 className="mb-2 capitalize text-xs uppercase tracking-wide text-gray-400">
                {type} ({cards.length})
              </h3>
              <ul className="space-y-2">
                {cards.map((card) => (
                  <li key={card.code} className="rounded bg-black/20 p-2">
                    <details className="group">
                      <summary className="flex items-center gap-2 text-sm font-semibold list-none cursor-pointer select-none">
                        {card.name}
                        <span className="ml-auto text-xs text-gray-500">
                          {card.class}
                          {card.cost == null ? "" : ` · cost ${card.cost}`}
                          {card.xp > 0 ? ` · xp ${card.xp}` : ""}
                        </span>
                      </summary>
                      <div className="mt-2 space-y-2 text-sm">
                        <p className="text-gray-400">{card.card_type}</p>
                        <div className="flex flex-wrap gap-1">
                          {SKILLS.map(([key, label]) => {
                            const value = card.icons[key];
                            if (!value) return null;
                            return (
                              <span
                                key={key}
                                className="rounded bg-arkham-border px-1.5 py-0 text-[10px] font-semibold uppercase tracking-tighter"
                              >
                                {label} {value}
                              </span>
                            );
                          })}
                          {card.traits.length > 0 && (
                            <span
                              className="rounded bg-black/20 px-1.5 py-0 text-[10px] text-gray-400 uppercase tracking-tighter"
                            >
                              {card.traits.join(" · ")}
                            </span>
                          )}
                        </div>
                        {card.text && (
                          <p className="whitespace-pre-line text-gray-300">
                            {card.text}
                          </p>
                        )}
                      </div>
                    </details>
                  </li>
                ))}
              </ul>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
