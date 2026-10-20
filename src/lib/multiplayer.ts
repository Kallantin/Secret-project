export type ClientMessage =
  | { type: 'create_room'; name?: string }
  | { type: 'join_room'; roomId: string; name?: string }
  | { type: 'leave_room' }
  | { type: 'action'; payload: unknown }
  | { type: 'ping' };

export type ServerMessage =
  | { type: 'welcome'; clientId: string; name: string }
  | { type: 'room_state'; roomId: string; players: PlayerInfo[] }
  | { type: 'player_joined'; player: PlayerInfo }
  | { type: 'player_left'; clientId: string }
  | { type: 'action'; clientId: string; payload: unknown }
  | { type: 'pong' }
  | { type: 'error'; message: string };

export type PlayerInfo = {
  clientId: string;
  name: string;
};

export type MultiplayerConnectionStatus = 'disconnected' | 'connecting' | 'connected';

export interface MultiplayerClientOptions {
  url?: string;
  onMessage?: (message: ServerMessage) => void;
  onStatusChange?: (status: MultiplayerConnectionStatus) => void;
}

export class MultiplayerClient {
  private ws: WebSocket | null = null;
  private status: MultiplayerConnectionStatus = 'disconnected';
  private clientId: string | null = null;
  private name: string | null = null;
  private roomId: string | null = null;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private readonly options: Required<MultiplayerClientOptions>;
  private readonly messageHandlers: Array<(message: ServerMessage) => void> = [];

  constructor(options: MultiplayerClientOptions = {}) {
    this.options = {
      url: options.url ?? 'ws://localhost:8787',
      onMessage: options.onMessage ?? (() => {}),
      onStatusChange: options.onStatusChange ?? (() => {}),
    };
    if (this.options.onMessage) {
      this.messageHandlers.push(this.options.onMessage);
    }
  }

  get isConnected(): boolean {
    return this.status === 'connected';
  }

  get currentClientId(): string | null {
    return this.clientId;
  }

  get currentRoomId(): string | null {
    return this.roomId;
  }

  get currentPlayerName(): string | null {
    return this.name;
  }

  get connectionStatus(): MultiplayerConnectionStatus {
    return this.status;
  }

  private setStatus(status: MultiplayerConnectionStatus): void {
    if (this.status === status) {
      return;
    }
    this.status = status;
    this.options.onStatusChange(status);
  }

  connect(): void {
    if (this.ws?.readyState === WebSocket.OPEN || this.ws?.readyState === WebSocket.CONNECTING) {
      return;
    }
    this.setStatus('connecting');
    try {
      this.ws = new WebSocket(this.options.url);
      this.ws.binaryType = 'arraybuffer';
      this.ws.onopen = () => {
        this.setStatus('connected');
      };
      this.ws.onmessage = (event: MessageEvent) => {
        try {
          const message: ServerMessage = JSON.parse(event.data as string);
          this.handleMessage(message);
        } catch {
          // ignore malformed messages
        }
      };
      this.ws.onclose = () => {
        this.ws = null;
        this.setStatus('disconnected');
        this.clientId = null;
        this.roomId = null;
        this.name = null;
        this.scheduleReconnect();
      };
      this.ws.onerror = () => {
        this.setStatus('disconnected');
      };
    } catch {
      this.setStatus('disconnected');
      this.scheduleReconnect();
    }
  }

  private scheduleReconnect(): void {
    if (this.reconnectTimer !== null) {
      clearTimeout(this.reconnectTimer);
    }
    this.reconnectTimer = setTimeout(() => {
      this.connect();
    }, 3000);
  }

  disconnect(): void {
    if (this.reconnectTimer !== null) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    this.setStatus('disconnected');
    this.clientId = null;
    this.roomId = null;
    this.name = null;
  }

  onMessage(handler: (message: ServerMessage) => void): () => void {
    this.messageHandlers.push(handler);
    return () => {
      const index = this.messageHandlers.indexOf(handler);
      if (index >= 0) {
        this.messageHandlers.splice(index, 1);
      }
    };
  }

  private handleMessage(message: ServerMessage): void {
    switch (message.type) {
      case 'welcome':
        this.clientId = message.clientId;
        this.name = message.name;
        break;
      case 'room_state':
        this.roomId = message.roomId;
        break;
      case 'player_joined':
        break;
      case 'player_left':
        break;
    }
    for (const handler of this.messageHandlers) {
      handler(message);
    }
  }

  private send(message: ClientMessage): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    }
  }

  createRoom(name?: string): void {
    this.send({ type: 'create_room', name });
  }

  joinRoom(roomId: string, name?: string): void {
    this.send({ type: 'join_room', roomId, name });
  }

  leaveRoom(): void {
    this.send({ type: 'leave_room' });
  }

  broadcastAction(payload: unknown): void {
    this.send({ type: 'action', payload });
  }

  ping(): void {
    this.send({ type: 'ping' });
  }
}
