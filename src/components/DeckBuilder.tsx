import { useMemo } from "react";
import { INVESTIGATORS } from "../data/investigators";
import { ALL_CARDS } from "../data/cards";
import { useGameStore } from "../store/gameStore";
function cardForCode(code: string) {
  return ALL_CARDS.find((card) => card.code === code);
}

export default function DeckBuilder() {
  const investigatorCode = useGameStore((state) => state.deckBuilderInvestigator);
  const deckCards = useGameStore((state) => state.deckBuilderCards);
  const setInvestigator = useGameStore((state) => state.setDeckBuilderInvestigator);
  const removeCard = useGameStore((state) => state.removeDeckCard);
  const setQuantity = useGameStore((state) => state.setDeckCardQuantity);
  const saveDeck = useGameStore((state) => state.saveDeckToInvestigator);

  const investigator = useMemo(
    () => INVESTIGATORS.find((inv) => inv.code === investigatorCode),
    [investigatorCode],
  );

  const deckWithDetails = useMemo(
    () =>
      deckCards
        .map((slot) => ({ ...slot, card: cardForCode(slot.code) }))
        .filter((entry) => entry.card !== undefined),
    [deckCards],
  );

  const totalCards = useMemo(
    () => deckWithDetails.reduce((sum, entry) => sum + entry.quantity, 0),
    [deckWithDetails],
  );

  const isValid = useMemo(
    () =>
      investigator !== undefined &&
      totalCards >= 30 &&
      totalCards <= 50 &&
      deckWithDetails.every((entry) => entry.quantity <= 2),
    [investigator, totalCards, deckWithDetails],
  );

  const handleInvestigatorChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    void setInvestigator(event.target.value || null);
  };

  const handleSave = () => {
    void saveDeck();
  };

  return (
    <div className="rounded-lg border border-arkham-border bg-arkham-panel p-4">
      <h2 className="mb-3 font-semibold text-arkham-accent">Deck Builder</h2>

      <div className="mb-4">
        <label className="block text-xs uppercase tracking-wide text-gray-400 mb-1">
          Investigator
        </label>
        <select
          className="w-full rounded bg-black/30 border border-arkham-border px-3 py-1.5 text-sm focus:outline-none focus:ring-1 focus:ring-arkham-accent"
          value={investigatorCode ?? ""}
          onChange={handleInvestigatorChange}
        >
          <option value="">Select an investigator</option>
          {INVESTIGATORS.map((inv) => (
            <option key={inv.code} value={inv.code}>
              {inv.name}
            </option>
          ))}
        </select>
      </div>

      {!investigatorCode ? (
        <p className="text-sm text-gray-400">
          Select an investigator to begin building their deck.
        </p>
      ) : deckWithDetails.length === 0 ? (
        <div className="text-center py-8">
          <p className="text-sm text-gray-400">No cards in deck yet.</p>
          <p className="text-xs text-gray-500 mt-1">
            Add cards from the Card Library panel.
          </p>
        </div>
      ) : (
        <>
          <div className="mb-3 flex items-center justify-between">
            <span className="text-sm font-medium">
              Deck: {totalCards} card{totalCards !== 1 ? "s" : ""}
            </span>
            <span
              className={`text-xs font-medium px-2 py-0.5 rounded ${
                isValid ? "bg-green-900/20 text-green-400 border border-green-500/50" : "bg-red-900/20 text-red-400 border border-red-500/50"
              }`}
            >
              {isValid ? "Valid" : "Invalid"}
            </span>
          </div>

          {!isValid && (
            <ul className="mb-3 text-xs text-red-400 list-disc pl-4 space-y-1">
              {!investigator && <li>Select an investigator.</li>}
              {totalCards < 30 && (
                <li>
                  Deck is below minimum ({totalCards}/30 cards).
                </li>
              )}
              {totalCards > 50 && (
                <li>
                  Deck exceeds maximum ({totalCards}/50 cards).
                </li>
              )}
              {deckWithDetails.some((entry) => entry.quantity > 2) && (
                <li>No more than 2 copies of any card.</li>
              )}
            </ul>
          )}

          <div className="overflow-auto max-h-64">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-arkham-border/50">
                  <th className="text-left text-xs uppercase tracking-wide text-gray-400 pb-2">
                    Card
                  </th>
                  <th className="text-right text-xs uppercase tracking-wide text-gray-400 pb-2">
                    Qty
                  </th>
                  <th className="text-right text-xs uppercase tracking-wide text-gray-400 pr-2 pb-2">
                    &
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-arkham-border/50">
                {deckWithDetails.map((entry) => {
                  const card = entry.card!;
                  return (
                    <tr key={entry.code} className="group hover:bg-black/10">
                      <td className="py-2">
                        <div className="font-medium text-gray-200">{card.name}</div>
<div className="text-xs text-gray-500">
  {card.card_type}{card.class ? ` · ${card.class}` : ""}
</div>
                      </td>
                      <td className="py-2 text-right">
                        <div className="flex items-center justify-end gap-1">
                          <button
                            type="button"
                            onClick={() => void setQuantity(entry.code, entry.quantity - 1)}
                            disabled={entry.quantity <= 1}
                            className="rounded border border-arkham-border px-1.5 py-0.5 text-xs text-gray-300 disabled:opacity-40 hover:bg-black/20"
                          >
                            −
                          </button>
                          <span className="w-6 text-center font-semibold">{entry.quantity}</span>
                          <button
                            type="button"
                            onClick={() => void setQuantity(entry.code, entry.quantity + 1)}
                            disabled={entry.quantity >= 2}
                            className="rounded border border-arkham-border px-1.5 py-0.5 text-xs text-gray-300 disabled:opacity-40 hover:bg-black/20"
                          >
                            +
                          </button>
                        </div>
                      </td>
                      <td className="py-2 text-right pr-2">
                        <button
                          type="button"
                          onClick={() => void removeCard(entry.code)}
                          className="text-xs text-red-400 hover:text-red-300"
                        >
                          Remove
                        </button>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>

          <div className="mt-4">
            <button
              type="button"
              onClick={handleSave}
              disabled={!isValid}
              className={`w-full rounded px-3 py-1.5 text-sm font-medium disabled:opacity-50 ${
                isValid
                  ? "bg-arkham-accent text-black hover:bg-arkham-accent/90"
                  : "border border-arkham-border text-gray-400"
              }`}
            >
              Save deck to {investigator?.name}
            </button>
          </div>
        </>
      )}
    </div>
  );
}
