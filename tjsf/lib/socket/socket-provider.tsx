'use client';

import { createContext, useEffect, useState, ReactNode } from 'react';
import { Socket } from 'socket.io-client';
import { socketClient } from './socket-client';
import { useAuthStore } from '@/lib/store/auth-store';
import { SOCKET_EVENTS_EMIT } from '@/lib/constants/socket-events';

interface SocketContextType {
  socket: Socket | null;
  isConnected: boolean;
}

export const SocketContext = createContext<SocketContextType>({
  socket: null,
  isConnected: false,
});

export function SocketProvider({ children }: { children: ReactNode }) {
  const [socket, setSocket] = useState<Socket | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const token = useAuthStore((state) => state.token);
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);
  const user = useAuthStore((state) => state.user);

  useEffect(() => {
    if (isAuthenticated && token && user) {
      const socketInstance = socketClient.connect(token);
      setSocket(socketInstance);

      const handleConnect = () => {
        setIsConnected(true);
        socketInstance.emit('identify', { user_id: user.id });
      };
      
      const handleDisconnect = () => {
        setIsConnected(false);
      };

      socketInstance.on('connect', handleConnect);
      socketInstance.on('disconnect', handleDisconnect);

      if (socketInstance.connected) {
        socketInstance.emit('identify', { user_id: user.id });
      }
      
      setIsConnected(socketInstance.connected);

      return () => {
        socketInstance.off('connect', handleConnect);
        socketInstance.off('disconnect', handleDisconnect);
      };
    } else {
      socketClient.disconnect();
      setSocket(null);
      setIsConnected(false);
    }
  }, [isAuthenticated, token, user?.id]);

  return (
    <SocketContext.Provider value={{ socket, isConnected }}>
      {children}
    </SocketContext.Provider>
  );
}
