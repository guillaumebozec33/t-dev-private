import { Ban, BanMemberInput, UnbanMemberInput } from '@/types';
import { apiClient } from '@/lib/api/client';

export const banMember = async (data: BanMemberInput) => {
  return apiClient.post<Ban>(`/servers/${data.serverId}/members/${data.memberId}/ban`, {
    duration_hours: data.duration_hours,
  });
};

export const unbanMember = async (data: UnbanMemberInput) => {
  return apiClient.delete(`/servers/${data.serverId}/bans/${data.userId}`);
};

export const getBans = async (serverId: string) => {
  return apiClient.get<Ban[]>(`/servers/${serverId}/bans`);
};
