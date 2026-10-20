import { useGameStore } from "../store/gameStore";

export default function AgendaPanel() {
  const game = useGameStore((state) => state.game);
  const lastPlaceDoomResult = useGameStore(
    (state) => state.lastPlaceDoomResult,
  );
  const busy = useGameStore((state) => state.busy);
  const placeDoom = useGameStore((state) => state.placeDoom);

  const agenda = game?.agendas[game?.current_agenda_index ?? 0];
  const act = game?.acts[game?.current_act_index ?? 0];

  if (!game || !agenda || !act) {
    return (
      <div className="rounded-lg border border-arkham-border bg-arkham-panel p-4">
        <h2 className="mb-3 font-semibold text-arkham-accent">Story</h2>
        <p className="text-sm text-gray-400">No active game.</p>
      </div>
    );
  }

  const doomPct = Math.max(
    0,
    Math.min(100, Math.round((agenda.doom / agenda.doom_threshold) * 100)),
  );
  const actPct = Math.max(
    0,
    Math.min(100, Math.round((act.progress / 3) * 100)),
  );

  return (
    <div className="rounded-lg border border-arkham-border bg-arkham-panel p-4">
      <h2 className="mb-3 font-semibold text-arkham-accent">Story</h2>

      <div className="mb-4">
        <div className="flex items-center justify-between mb-2">
          <h3 className="font-semibold text-gray-200">
            Agenda: {agenda.card.name}
          </h3>
          <span className="text-xs text-gray-400">
            {game.game_status === "won"
              ? "VICTORY"
              : game.game_status === "lost"
              ? "DEFEAT"
              : "Ongoing"}
          </span>
        </div>
        <p className="text-[10px] text-gray-400 mb-2">
          {agenda.card.text}
        </p>
        <div className="w-full bg-gray-700 rounded-full h-1.5 mb-1">
          <div
            className="bg-red-500 rounded-full h-1.5"
            style={{ width: `${doomPct}%` }}
          />
        </div>
        <div className="flex justify-between text-[10px] text-gray-500">
          <span>Doom: {agenda.doom}</span>
          <span>{agenda.doom_threshold}</span>
        </div>
      </div>

      <div className="mb-4">
        <div className="flex items-center justify-between mb-2">
          <h3 className="font-semibold text-gray-200">
            Act: {act.card.name}
          </h3>
        </div>
        <p className="text-[10px] text-gray-400 mb-2">{act.card.text}</p>
        <div className="w-full bg-gray-700 rounded-full h-1.5 mb-1">
          <div
            className="bg-arkham-accent rounded-full h-1.5"
            style={{ width: `${actPct}%` }}
          />
        </div>
        <div className="flex justify-between text-[10px] text-gray-500">
          <span>Progress: {act.progress}</span>
          <span>3</span>
        </div>
      </div>

      <div className="mb-3">
        <button
          type="button"
          onClick={() => void placeDoom(1)}
          disabled={
            busy ||
            game.game_status !== "ongoing" ||
            game.phase !== "mythos"
          }
          className="w-full rounded border border-arkham-border px-3 py-1.5 text-sm text-gray-300 disabled:opacity-40 hover:bg-black/20"
        >
          Place Doom (1)
        </button>
      </div>

      {lastPlaceDoomResult && (
        <div className="mt-3 pt-3 border-t border-arkham-border/50">
          <p className="text-xs uppercase tracking-wide text-gray-400 mb-1">
            Last Doom
          </p>
          <div className="text-sm text-red-400">
            {lastPlaceDoomResult.advanced
              ? `Agenda advanced! Doom reset.`
              : `Doom: {lastPlaceDoomResult.doom}/{lastPlaceDoomResult.doom_threshold}`}
          </div>
        </div>
      )}
    </div>
  );
}
