'use client';

import { useEffect } from 'react';
import { useQueryClient, useMutation } from '@tanstack/react-query';
import { useSocket } from '@/lib/socket/use-socket';
import { SOCKET_EVENTS_LISTEN } from '@/lib/constants/socket-events';
import { Member, StoredUserStatus, UserStatus } from '@/types';
import { updateMyStatus } from '@/lib/api/endpoints';
import {useSelectedServer} from "@/hooks";

export function useUserStatus() {
  const { socket, isConnected } = useSocket();
  const queryClient = useQueryClient();
  const {selectedServer} = useSelectedServer();
  const serverId = selectedServer?.id

  const updateStatusMutation = useMutation({
    mutationFn: updateMyStatus,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['me'] });
    },
  });

  const updateStatus = (status: StoredUserStatus) => {
    updateStatusMutation.mutate(status);
  };

  useEffect(() => {
    if (!socket || !isConnected || !serverId) return;

    const handleUserStatusChanged = (data: { 
      server_id: string; 
      user_id: string; 
      status: UserStatus 
    }) => {
      if (data.server_id === serverId) {
          if (data.status === "dnd"){
              data.status = "donotdisturb"
          }
        queryClient.setQueryData<Member[]>(
          ['members', serverId],
          (old = []) => old.map((member) => 
            member.user_id === data.user_id 
              ? { ...member, status: data.status }
              : member
          )
        );
      }
    };

    socket.on(SOCKET_EVENTS_LISTEN.USER_STATUS_CHANGED, handleUserStatusChanged);

    return () => {
      socket.off(SOCKET_EVENTS_LISTEN.USER_STATUS_CHANGED, handleUserStatusChanged);
    };
  }, [socket, isConnected, serverId, queryClient]);

  return { updateStatus, isUpdating: updateStatusMutation.isPending };
}
