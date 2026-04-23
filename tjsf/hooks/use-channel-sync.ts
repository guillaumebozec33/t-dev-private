'use client';

import { useEffect } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { useSocket } from '@/lib/socket/use-socket';
import { SOCKET_EVENTS_LISTEN } from '@/lib/constants/socket-events';
import { Channel } from '@/types';
import {useSelectedServer} from "@/hooks";

export function useChannelSync() {
    const { selectedServer } = useSelectedServer();
  const { socket, isConnected } = useSocket();
  const queryClient = useQueryClient();

  useEffect(() => {
    if (!socket || !isConnected || !selectedServer?.id) return;

    const handleChannelCreated = (channel: Channel) => {
      if (channel.server_id === selectedServer?.id) {
        queryClient.setQueryData<Channel[]>(
          ['channels', selectedServer?.id],
          (old = []) => [...old, channel]
        );
      }
    };

    const handleChannelUpdated = (channel: Channel) => {
      if (channel.server_id === selectedServer?.id) {
        queryClient.setQueryData<Channel[]>(
          ['channels', selectedServer?.id],
          (old = []) => old.map((c) => (c.id === channel.id ? channel : c))
        );
      }
    };

    const handleChannelDeleted = (data: { id: string; server_id: string }) => {
      if (data.server_id === selectedServer?.id) {
        queryClient.setQueryData<Channel[]>(
          ['channels', selectedServer?.id],
          (old = []) => old.filter((c) => c.id !== data.id)
        );
      }
    };

    socket.on(SOCKET_EVENTS_LISTEN.CHANNEL_CREATED, handleChannelCreated);
    socket.on(SOCKET_EVENTS_LISTEN.CHANNEL_UPDATED, handleChannelUpdated);
    socket.on(SOCKET_EVENTS_LISTEN.CHANNEL_DELETED, handleChannelDeleted);

    return () => {
      socket.off(SOCKET_EVENTS_LISTEN.CHANNEL_CREATED, handleChannelCreated);
      socket.off(SOCKET_EVENTS_LISTEN.CHANNEL_UPDATED, handleChannelUpdated);
      socket.off(SOCKET_EVENTS_LISTEN.CHANNEL_DELETED, handleChannelDeleted);
    };
  }, [socket, isConnected, selectedServer?.id, queryClient]);
}
