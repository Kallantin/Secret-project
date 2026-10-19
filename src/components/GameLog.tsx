export default function GameLog({ entries }: { entries: string[] }) {
  const ordered = [...entries].reverse();

  return (
    <div className="rounded-lg border border-arkham-border bg-arkham-panel p-4">
      <h2 className="mb-3 font-semibold text-arkham-accent">Log</h2>
      {ordered.length === 0 ? (
        <p className="text-sm text-gray-400">No events yet.</p>
      ) : (
        <ol className="flex flex-col gap-1 text-sm text-gray-300">
          {ordered.map((entry, index) => (
            <li key={`${index}-${entry}`}>{entry}</li>
          ))}
        </ol>
      )}
    </div>
  );
}
