import { useMemo, useState } from "react";
import { INVESTIGATORS } from "../data/investigators";
import type { CardTemplate } from "../data/cards";
import { ENCOUNTER_CARDS, PLAYER_CARDS } from "../data/cards";
import { useGameStore } from "../store/gameStore";

const CARDS: CardTemplate[] = [...PLAYER_CARDS, ...ENCOUNTER_CARDS];

const SKILLS: readonly [keyof CardTemplate["icons"], string][] = [
  ["willpower", "W"],
  ["intellect", "I"],
  ["combat", "C"],
  ["agility", "A"],
  ["wild", "*"],
];

const EXPANSIONS: { code: string; name: string }[] = [
  { code: "core", name: "Core Set" },
];

// Taboo list: card codes that are banned from deckbuilding
const TABOO_LIST: string[] = [];

export default function CardLibrary() {
  const [query, setQuery] = useState("");
  const [expansionFilter, setExpansionFilter] = useState<string | null>(null);
  const deckBuilderInvestigator = useGameStore(
    (state) => state.deckBuilderInvestigator,
  );
  const deckBuilderCards = useGameStore((state) => state.deckBuilderCards);
  const addDeckCard = useGameStore((state) => state.addDeckCard);
  const removeDeckCard = useGameStore((state) => state.removeDeckCard);

  const investigator = useMemo(
    () => INVESTIGATORS.find((inv) => inv.code === deckBuilderInvestigator),
    [deckBuilderInvestigator],
  );

  const deckQuantity = useMemo(
    () =>
      deckBuilderCards.reduce(
        (map, slot) => {
          map.set(slot.code, slot.quantity);
          return map;
        },
        new Map<string, number>(),
      ),
    [deckBuilderCards],
  );

  const groups = useMemo(() => {
    const q = query.toLowerCase();
    const map = new Map<string, CardTemplate[]>();
    for (const card of CARDS) {
      if (investigator) {
        const allowed = card.class === "neutral" || card.class === investigator.class;
        if (!allowed) continue;
      }
      // Expansion filter
      if (expansionFilter) {
        const packCode = card.raw.pack_code;
        if (packCode !== expansionFilter) continue;
      }
      // Taboo filter
      if (TABOO_LIST.includes(card.code)) continue;
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
  }, [query, investigator, expansionFilter]);

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
      <div className="flex gap-2 mt-2">
        <label className="text-xs text-gray-400 uppercase tracking-wide">Expansion</label>
        <select
          className="rounded bg-arkham-panel border border-arkham-border px-3 py-1.5 text-sm focus:outline-none focus:ring-1 focus:ring-arkham-accent"
          value={expansionFilter ?? ""}
          onChange={(e) => setExpansionFilter(e.target.value || null)}
        >
          <option value="">All</option>
          {EXPANSIONS.map((exp) => (
            <option key={exp.code} value={exp.code}>
              {exp.name}
            </option>
          ))}
        </select>
      </div>
      {deckBuilderInvestigator && !investigator ? (
        <p className="text-sm text-gray-400">Unknown investigator.</p>
      ) : groups.length === 0 ? (
        <p className="text-sm text-gray-400">No matching cards.</p>
      ) : (
        <ul className="divide-y divide-arkham-border/50">
          {groups.map(([type, cards]) => (
            <li key={type} className="p-3">
              <h3 className="mb-2 capitalize text-xs uppercase tracking-wide text-gray-400">
                {type} ({cards.length})
              </h3>
              <ul className="space-y-2">
                {cards.map((card) => {
                  const qty = deckQuantity.get(card.code) ?? 0;
                  const isTaboo = TABOO_LIST.includes(card.code);
                  return (
                    <li key={card.code} className={`rounded p-2 ${isTaboo ? "bg-red-900/20 border border-red-900/50" : "bg-black/20"}`}>
                      <details className="group">
                        <summary className="flex items-center gap-2 text-sm font-semibold list-none cursor-pointer select-none">
                          {card.name}
                          <span className="ml-auto flex items-center gap-1 text-xs text-gray-500">
                            {deckBuilderInvestigator ? (
                              <>
                                <button
                                  type="button"
                                  onClick={() => void removeDeckCard(card.code)}
                                  disabled={qty <= 0 || !deckBuilderInvestigator || isTaboo}
                                  className="rounded border border-arkham-border px-1 py-0 text-[10px] text-gray-400 disabled:opacity-40 hover:bg-black/20"
                                >
                                  −
                                </button>
                                <span className="w-4 text-center font-semibold">
                                  {qty}
                                </span>
                                <button
                                  type="button"
                                  onClick={() => void addDeckCard(card.code)}
                                  disabled={qty >= 2 || !deckBuilderInvestigator || isTaboo}
                                  className="rounded border border-arkham-border px-1 py-0 text-[10px] text-gray-400 disabled:opacity-40 hover:bg-black/20"
                                >
                                  +
                                </button>
                              </>
                            ) : (
                              <span className="text-[10px] text-gray-500 italic">
                                Select investigator
                              </span>
                            )}
                            {card.class}
                            {card.cost == null ? "" : ` · cost ${card.cost}`}
                            {card.xp > 0 ? ` · xp ${card.xp}` : ""}
                            {isTaboo && " · taboo"}
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
                  );
                })}
              </ul>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
