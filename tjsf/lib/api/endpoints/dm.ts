import { apiClient } from '@/lib/api/client';
import { DmConversation, DmMessage } from '@/types';

export const openConversation = async (userId: string): Promise<DmConversation> => {
  return apiClient.post('/dm/conversations', { user_id: userId });
};

export const getConversations = async (): Promise<DmConversation[]> => {
  return apiClient.get('/dm/conversations');
};

export const getDmMessages = async (conversationId: string): Promise<DmMessage[]> => {
  const messages = await apiClient.get<DmMessage[]>(`/dm/conversations/${conversationId}/messages`);
  return messages.reverse();
};

export const sendDm = async (conversationId: string, content: string): Promise<DmMessage> => {
  return apiClient.post(`/dm/conversations/${conversationId}/messages`, { content });
};
