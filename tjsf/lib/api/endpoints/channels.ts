import {EditChannel,CreateChannel,Channel} from '@/types'
import { apiClient } from '@/lib/api/client';

export const getChannels = async (serverId:string)=>{
    return apiClient.get<Channel[]>(`/servers/${serverId}/channels`)
};

export const addChannels = async (data:CreateChannel)=>{
    console.log(data)
    return apiClient.post(`/servers/${data.server_id}/channels`,data)
};


export const updateChannel = async (data:EditChannel)=>{
    return apiClient.put(`/channels/${data.id}`, data)
};


export const deleteChannel= async (channelId:string) =>{
    return apiClient.delete(`/channels/${channelId}`)
};