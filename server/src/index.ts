import { createServer } from 'node:http';
import type { IncomingMessage, ServerResponse } from 'node:http';
import { randomUUID } from 'node:crypto';
import { WebSocketServer, WebSocket } from 'ws';
import type { RawData } from 'ws';

const PORT = Number.parseInt(process.env.PORT ?? '8787', 10);
const HEARTBEAT_INTERVAL_MS = 30_000;
const ROOM_CODE_LENGTH = 6;
const ROOM_ALPHABET = 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789';

interface PlayerInfo {
  clientId: string;
  name: string;
}

interface Client {
  id: string;
  name: string;
  roomId: string | null;
  alive: boolean;
  ws: WebSocket;
}

type ClientMessage =
  | { type: 'create_room'; name?: string }
  | { type: 'join_room'; roomId: string; name?: string }
  | { type: 'leave_room' }
  | { type: 'action'; payload: unknown }
  | { type: 'ping' };

type ServerMessage =
  | { type: 'welcome'; clientId: string; name: string }
  | { type: 'room_state'; roomId: string; players: PlayerInfo[] }
  | { type: 'player_joined'; player: PlayerInfo }
  | { type: 'player_left'; clientId: string }
  | { type: 'action'; clientId: string; payload: unknown }
  | { type: 'pong' }
  | { type: 'error'; message: string };

class Room {
  readonly id: string;
  readonly clients = new Map<string, Client>();

  constructor(id: string) {
    this.id = id;
  }

  get players(): PlayerInfo[] {
    return [...this.clients.values()].map((client) => ({
      clientId: client.id,
      name: client.name,
    }));
  }

  get isEmpty(): boolean {
    return this.clients.size === 0;
  }

  broadcast(message: ServerMessage, except?: string): void {
    const data = JSON.stringify(message);
    for (const client of this.clients.values()) {
      if (client.id !== except && client.ws.readyState === WebSocket.OPEN) {
        client.ws.send(data);
      }
    }
  }
}

const clients = new Map<WebSocket, Client>();
const rooms = new Map<string, Room>();

function send(ws: WebSocket, message: ServerMessage): void {
  if (ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify(message));
  }
}

function sendError(ws: WebSocket, message: string): void {
  send(ws, { type: 'error', message });
}

function generateRoomCode(): string {
  let code = '';
  for (let index = 0; index < ROOM_CODE_LENGTH; index += 1) {
    const offset = Math.floor(Math.random() * ROOM_ALPHABET.length);
    code += ROOM_ALPHABET[offset] ?? 'A';
  }
  if (rooms.has(code)) {
    return generateRoomCode();
  }
  return code;
}

function enterRoom(client: Client, room: Room): void {
  leaveRoom(client);
  client.roomId = room.id;
  room.clients.set(client.id, client);
  room.broadcast(
    { type: 'player_joined', player: { clientId: client.id, name: client.name } },
    client.id,
  );
  room.broadcast({ type: 'room_state', roomId: room.id, players: room.players });
}

function leaveRoom(client: Client): void {
  if (client.roomId === null) {
    return;
  }
  const room = rooms.get(client.roomId);
  client.roomId = null;
  if (room === undefined) {
    return;
  }
  room.clients.delete(client.id);
  room.broadcast({ type: 'player_left', clientId: client.id });
  if (room.isEmpty) {
    rooms.delete(room.id);
    return;
  }
  room.broadcast({ type: 'room_state', roomId: room.id, players: room.players });
}

function handleMessage(client: Client, raw: RawData): void {
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw.toString());
  } catch {
    sendError(client.ws, 'invalid json');
    return;
  }

  if (typeof parsed !== 'object' || parsed === null || !('type' in parsed)) {
    sendError(client.ws, 'invalid message');
    return;
  }

  const message = parsed as ClientMessage;

  switch (message.type) {
    case 'create_room': {
      if (typeof message.name === 'string' && message.name.length > 0) {
        client.name = message.name;
      }
      const room = new Room(generateRoomCode());
      rooms.set(room.id, room);
      enterRoom(client, room);
      break;
    }
    case 'join_room': {
      const room = rooms.get(message.roomId);
      if (room === undefined) {
        sendError(client.ws, `room not found: ${message.roomId}`);
        return;
      }
      if (typeof message.name === 'string' && message.name.length > 0) {
        client.name = message.name;
      }
      enterRoom(client, room);
      break;
    }
    case 'leave_room': {
      leaveRoom(client);
      break;
    }
    case 'action': {
      if (client.roomId === null) {
        sendError(client.ws, 'not in a room');
        return;
      }
      const room = rooms.get(client.roomId);
      room?.broadcast({ type: 'action', clientId: client.id, payload: message.payload }, client.id);
      break;
    }
    case 'ping': {
      send(client.ws, { type: 'pong' });
      break;
    }
    default: {
      sendError(client.ws, 'unknown message type');
    }
  }
}

const server = createServer((request: IncomingMessage, response: ServerResponse) => {
  if (request.method === 'GET') {
    response.writeHead(200, { 'content-type': 'application/json' });
    response.end(JSON.stringify({ status: 'ok', rooms: rooms.size, clients: clients.size }));
    return;
  }
  response.writeHead(405, { 'content-type': 'application/json' });
  response.end(JSON.stringify({ status: 'error', message: 'method not allowed' }));
});

const wss = new WebSocketServer({ server });

wss.on('connection', (ws: WebSocket) => {
  const client: Client = {
    id: randomUUID(),
    name: `Player-${clients.size + 1}`,
    roomId: null,
    alive: true,
    ws,
  };
  clients.set(ws, client);
  send(ws, { type: 'welcome', clientId: client.id, name: client.name });

  ws.on('message', (raw: RawData) => {
    handleMessage(client, raw);
  });

  ws.on('pong', () => {
    client.alive = true;
  });

  ws.on('close', () => {
    leaveRoom(client);
    clients.delete(ws);
  });

  ws.on('error', () => {
    clients.delete(ws);
  });
});

const heartbeat = setInterval(() => {
  for (const [ws, client] of clients) {
    if (!client.alive) {
      ws.terminate();
      clients.delete(ws);
      continue;
    }
    client.alive = false;
    ws.ping();
  }
}, HEARTBEAT_INTERVAL_MS);

wss.on('close', () => {
  clearInterval(heartbeat);
});

server.listen(PORT, () => {
  console.log(`arkham server listening on :${PORT}`);
});
