import {SendMessage, Message} from '@/types'
import { apiClient } from '@/lib/api/client';


export const getMessages = async (channelId: string, before?: string): Promise<Message[]> => {
    const params = new URLSearchParams({ limit: '50' });
    if (before) params.append('before', before);

    return apiClient.get<Message[]>(`/channels/${channelId}/messages?${params}`);
};

export const sendMessage = async (data:SendMessage)=>{
    return apiClient.post(`/channels/${data.channel_id}/messages`,data);
}

export const deleteMessage = async (messageId:string)=>{
    return apiClient.delete(`/messages/${messageId}`);
}
export const editMessage = async (messageId: string, content: string) => {
    return apiClient.patch(`/messages/${messageId}`, { content });
};