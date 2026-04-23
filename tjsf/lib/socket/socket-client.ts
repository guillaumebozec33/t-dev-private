import { io, Socket } from 'socket.io-client';
import {config} from '@/lib/constants/config'


class SocketClient {
  private socket: Socket | null = null;
  private token: string | null = null;

  connect(token?: string) {
    if (this.socket?.connected) {
      return this.socket;
    }

    if (token) {
      this.token = token;
    }

    this.socket = io(config.wsUrl, {
      transports: ['websocket', 'polling'],
      auth: {
        token: this.token,
      },
      reconnection: true,
      reconnectionDelay: 1000,
      reconnectionDelayMax: 5000,
      reconnectionAttempts: Infinity,
      timeout: 20000,
    });

    this.socket.on('connect', () => {
      console.log('Socket connected:', this.socket?.id);
    });

    this.socket.on('disconnect', (reason) => {
      console.log('Socket disconnected:', reason);
    });

    this.socket.on('connect_error', (error) => {
      console.error('Socket connection error:', error);
    });

    return this.socket;
  }

  disconnect() {
    if (this.socket) {
      this.socket.disconnect();
      this.socket = null;
    }
  }
}

export const socketClient = new SocketClient();
