import { useMultiplayerStore } from "../store/multiplayerStore";
import type { PlayerInfo } from "../lib/multiplayer";
import { useState } from "react";

export default function RoomLobby() {
  const {
    status,
    roomId,
    clientId,
    playerName,
    players,
    error,
    connect,
    disconnect,
    createRoom,
    joinRoom,
    leaveRoom,
    setPlayerName,
    clearError,
  } = useMultiplayerStore((state) => ({
    status: state.status,
    roomId: state.roomId,
    clientId: state.clientId,
    playerName: state.playerName,
    players: state.players,
    error: state.error,
    connect: state.connect,
    disconnect: state.disconnect,
    createRoom: state.createRoom,
    joinRoom: state.joinRoom,
    leaveRoom: state.leaveRoom,
    setPlayerName: state.setPlayerName,
    clearError: state.clearError,
  }));

  const isConnected = status === "connected";
  const inRoom = roomId !== null;
  const [roomCode, setRoomCode] = useState("");

  const notInRoomPanel = isConnected && !inRoom ? (
    <div className="space-y-2">
      <div className="flex gap-2">
        <input
          type="text"
          placeholder="Your name"
          value={playerName}
          onChange={(e) => setPlayerName(e.target.value)}
          className="flex-1 rounded bg-black/30 border border-arkham-border px-3 py-1.5 text-sm focus:outline-none focus:ring-1 focus:ring-arkham-accent"
        />
        <button
          type="button"
          onClick={() => createRoom(playerName || undefined)}
          disabled={!playerName}
          className="rounded bg-arkham-accent px-3 py-1.5 text-sm font-medium text-black disabled:opacity-50"
        >
          Create Room
        </button>
      </div>
      <div className="flex gap-2">
        <input
          type="text"
          placeholder="Room code"
          value={roomCode}
          onChange={(e) => setRoomCode(e.target.value)}
          className="flex-1 rounded bg-black/30 border border-arkham-border px-3 py-1.5 text-sm focus:outline-none focus:ring-1 focus:ring-arkham-accent"
        />
        <button
          type="button"
          onClick={() => joinRoom(roomCode.toUpperCase(), playerName || undefined)}
          disabled={!roomCode || !playerName}
          className="rounded bg-arkham-accent px-3 py-1.5 text-sm font-medium text-black disabled:opacity-50"
        >
          Join
        </button>
      </div>
    </div>
  ) : null;

  const inRoomPanel = inRoom ? (
    <div className="space-y-2">
      <div className="flex items-center justify-between mb-2">
        <span className="text-sm font-semibold text-gray-200">
          Room: {roomId}
        </span>
        <button
          type="button"
          onClick={() => leaveRoom()}
          className="rounded border border-arkham-border px-3 py-1.5 text-sm text-gray-300 hover:bg-black/20"
        >
          Leave
        </button>
      </div>
      <p className="text-xs uppercase tracking-wide text-gray-400 mb-1">
        Players ({players.length})
      </p>
      <ul className="space-y-1">
        {players.map((player: PlayerInfo) => (
          <li
            key={player.clientId}
            className={`flex items-center gap-2 text-sm ${
              player.clientId === clientId
                ? "text-arkham-accent font-semibold"
                : "text-gray-300"
            }`}
          >
            {player.name}{player.clientId === clientId && " (you)"}
          </li>
        ))}
      </ul>
    </div>
  ) : null;

  return (
    <div className="rounded-lg border border-arkham-border bg-arkham-panel p-4">
      <h2 className="mb-3 font-semibold text-arkham-accent">Multiplayer</h2>

      <div className="mb-3 flex items-center gap-2">
        <span
          className={`text-xs font-semibold uppercase tracking-wide ${
            status === "connected"
              ? "text-green-400"
              : status === "connecting"
              ? "text-yellow-400"
              : "text-red-400"
          }`}
        >
          {status}
        </span>
        {!isConnected && (
          <button
            type="button"
            onClick={() => connect()}
            className="rounded border border-arkham-border px-3 py-1.5 text-sm text-gray-300 hover:bg-black/20"
          >
            Connect
          </button>
        )}
        {isConnected && (
          <button
            type="button"
            onClick={() => disconnect()}
            className="rounded border border-arkham-border px-3 py-1.5 text-sm text-gray-300 hover:bg-black/20"
          >
            Disconnect
          </button>
        )}
      </div>

      {error && (
        <div className="mb-3 p-2 rounded bg-red-900/20 text-xs text-red-400">
          {error}
          <button
            type="button"
            onClick={() => clearError()}
            className="ml-2 text-xs underline"
          >
            dismiss
          </button>
        </div>
      )}

      {notInRoomPanel}
      {inRoomPanel}
    </div>
  );
}
