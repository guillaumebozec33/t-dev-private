'use client';

import { useEffect, useState } from 'react';
import { useSocket } from '@/lib/socket/use-socket';
import { Member, UserStatus } from '@/types';
import {useAuthStore} from '@/lib/store/auth-store';

export function useMemberPresence(members: Member[]) {
  const { socket, isConnected } = useSocket();
  const [onlineUserIds, setOnlineUserIds] = useState<Set<string>>(new Set());
  const me = useAuthStore((state) => state.user);

  useEffect(() => {
    const initialOnline = new Set<string>();
    members.forEach((member) => {
      if (member.status !== 'offline') {
        initialOnline.add(member.user_id);
      }
    });
    if (me) {
      initialOnline.add(me.id);
    }
    setOnlineUserIds(initialOnline);
  }, [members, me]);

  useEffect(() => {
    if (!socket || !isConnected) return;

    const handleUserConnected = (data: { user_id: string }) => {
      setOnlineUserIds((prev) => new Set(prev).add(data.user_id));
    };

    const handleUserDisconnected = (data: { user_id: string }) => {
      setOnlineUserIds((prev) => {
        const next = new Set(prev);
        next.delete(data.user_id);
        return next;
      });
    };

    socket.on('user_connected', handleUserConnected);
    socket.on('user_disconnected', handleUserDisconnected);

    return () => {
      socket.off('user_connected', handleUserConnected);
      socket.off('user_disconnected', handleUserDisconnected);
    };
  }, [socket, isConnected]);


  return members.map((member) => {
    const isOnline = onlineUserIds.has(member.user_id);
    const isInvisible = member.status === 'invisible';
    
    return {
      ...member,
      displayStatus: (isOnline && !isInvisible) 
        ? member.status 
        : 'offline' as UserStatus,
    };
  });
}
