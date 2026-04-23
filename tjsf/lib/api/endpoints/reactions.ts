import { apiClient } from '@/lib/api/client';
import { Reaction } from '@/types/reaction';

export const toggleReaction = async (messageId: string, emoji: string): Promise<Reaction[]> => {
  return apiClient.put(`/messages/${messageId}/reactions`, { emoji });
};

export const toggleDmReaction = async (messageId: string, emoji: string): Promise<Reaction[]> => {
  return apiClient.put(`/dm/messages/${messageId}/reactions`, { emoji });
};

export const getReactions = async (messageId: string): Promise<Reaction[]> => {
  return apiClient.get(`/messages/${messageId}/reactions`);
};
