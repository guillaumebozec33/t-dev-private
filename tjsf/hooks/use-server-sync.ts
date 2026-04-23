'use client';

import { useEffect } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { useSocket } from '@/lib/socket/use-socket';
import { SOCKET_EVENTS_LISTEN } from '@/lib/constants/socket-events';
import { Server } from '@/types';

export function useServerSync() {
  const { socket, isConnected } = useSocket();
  const queryClient = useQueryClient();

  useEffect(() => {
    if (!socket || !isConnected) return;

    const handleServerUpdated = (server: Server) => {
      queryClient.setQueryData<Server[]>(
        ['servers'],
        (old = []) => old.map((s) => (s.id === server.id ? { ...s, ...server } : s))
      );
    };

    socket.on(SOCKET_EVENTS_LISTEN.SERVER_UPDATED, handleServerUpdated);
    return () => {
      socket.off(SOCKET_EVENTS_LISTEN.SERVER_UPDATED, handleServerUpdated);
    };
  }, [socket, isConnected, queryClient]);
}
