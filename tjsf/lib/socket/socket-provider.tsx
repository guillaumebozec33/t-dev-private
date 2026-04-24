'use client';

import { createContext, useEffect, useState, ReactNode } from 'react';
import { Socket } from 'socket.io-client';
import { socketClient } from './socket-client';
import { useAuthStore } from '@/lib/store/auth-store';

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
  const userId = useAuthStore((state) => state.user?.id);

  useEffect(() => {
    if (isAuthenticated && token && userId) {
      const socketInstance = socketClient.connect(token);
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setSocket(socketInstance);

      const handleConnect = () => {
        setIsConnected(true);
        socketInstance.emit('identify', { user_id: userId });
      };
      
      const handleDisconnect = () => {
        setIsConnected(false);
      };

      socketInstance.on('connect', handleConnect);
      socketInstance.on('disconnect', handleDisconnect);

      if (socketInstance.connected) {
        socketInstance.emit('identify', { user_id: userId });
      }

      return () => {
        socketInstance.off('connect', handleConnect);
        socketInstance.off('disconnect', handleDisconnect);
      };
    } else {
      socketClient.disconnect();
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setSocket(null);
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setIsConnected(false);
    }
  }, [isAuthenticated, token, userId]);

  return (
    <SocketContext.Provider value={{ socket, isConnected }}>
      {children}
    </SocketContext.Provider>
  );
}
