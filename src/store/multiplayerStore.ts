import { create } from 'zustand';
import { MultiplayerClient, type PlayerInfo } from '../lib/multiplayer';

export type MultiplayerStatus = 'disconnected' | 'connecting' | 'connected';

interface MultiplayerStore {
  client: MultiplayerClient | null;
  status: MultiplayerStatus;
  roomId: string | null;
  clientId: string | null;
  playerName: string;
  players: PlayerInfo[];
  error: string | null;
  connect: (url?: string) => void;
  disconnect: () => void;
  createRoom: (name?: string) => void;
  joinRoom: (roomId: string, name?: string) => void;
  leaveRoom: () => void;
  broadcastAction: (type: string, payload: unknown) => void;
  setPlayerName: (name: string) => void;
  clearError: () => void;
  onRemoteAction: ((action: { type: string; payload: unknown; clientId: string | null }) => void) | null;
  setRemoteActionHandler: (handler: (action: { type: string; payload: unknown; clientId: string | null }) => void) => void;
}

export const useMultiplayerStore = create<MultiplayerStore>((set, get) => ({
  client: null,
  status: 'disconnected',
  roomId: null,
  clientId: null,
  playerName: '',
  players: [],
  error: null,
  onRemoteAction: null,

  connect: (url?: string) => {
    const { client: existingClient } = get();
    if (existingClient?.isConnected) {
      return;
    }
    const client = new MultiplayerClient({
      url,
      onStatusChange: (status) => {
        set({ status });
      },
      onMessage: (message) => {
        switch (message.type) {
          case 'welcome':
            set({
              clientId: message.clientId,
              playerName: message.name,
            });
            break;
          case 'room_state':
            set({ roomId: message.roomId, players: message.players });
            // Request state sync if we don't have a game yet
            get().onRemoteAction?.({
              type: 'request_state_sync',
              payload: null,
              clientId: null,
            });
            break;
          case 'player_joined':
            set((state) => ({
              players: [...state.players, message.player],
            }));
            break;
          case 'player_left':
            set((state) => ({
              players: state.players.filter((p) => p.clientId !== message.clientId),
            }));
            break;
          case 'action':
            get().onRemoteAction?.({
              type: message.type,
              payload: message.payload,
              clientId: message.clientId,
            });
            break;
          case 'game_state_sync':
            get().onRemoteAction?.({
              type: 'game_state_sync',
              payload: message.gameState,
              clientId: null,
            });
            break;
          case 'error':
            set({ error: message.message });
            break;
          default:
            break;
        }
      },
    });
    client.connect();
    set({ client, error: null });
  },

  disconnect: () => {
    const { client } = get();
    if (client) {
      client.disconnect();
    }
    set({
      client: null,
      status: 'disconnected',
      roomId: null,
      clientId: null,
      playerName: '',
      players: [],
      error: null,
    });
  },

  createRoom: (name?: string) => {
    const { client } = get();
    if (!client?.isConnected) {
      set({ error: 'Not connected to server' });
      return;
    }
    client.createRoom(name);
  },

  joinRoom: (roomId: string, name?: string) => {
    const { client } = get();
    if (!client?.isConnected) {
      set({ error: 'Not connected to server' });
      return;
    }
    client.joinRoom(roomId, name);
  },

  leaveRoom: () => {
    const { client } = get();
    if (!client) {
      return;
    }
    client.leaveRoom();
    set({ roomId: null, players: [], error: null });
  },

  broadcastAction: (type: string, payload: unknown) => {
    const { client } = get();
    if (!client?.isConnected) {
      return;
    }
    client.broadcastAction({ type, payload });
  },

  setPlayerName: (name: string) => {
    set({ playerName: name });
  },

  clearError: () => {
    set({ error: null });
  },

  setRemoteActionHandler: (handler: (action: { type: string; payload: unknown; clientId: string | null }) => void) => {
    set({ onRemoteAction: handler });
  },
}));
