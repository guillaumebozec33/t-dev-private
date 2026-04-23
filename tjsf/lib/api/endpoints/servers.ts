import {EditServer,CreateServer, Invitation,InvitationOutput, CreateInvitationInput,Server,KickMemberInput} from '@/types'
import { apiClient } from '@/lib/api/client';

export const getServers=async()=>{
    return apiClient.get<Server[]>("/servers");
}

export const addServers = async (data:CreateServer)=>{
    return apiClient.post<Server>('/servers',data);
}

export const generateCode = async (data:CreateInvitationInput)=>{
    return apiClient.post<InvitationOutput>(`/servers/${data.server_id}/invitations`,data);
}

export const joinServer = async (data:Invitation)=>{
    return apiClient.post<Server>("/servers/join",data);
}

export const updateServer = async(data:EditServer)=>{
    return apiClient.put(`/servers/${data.id}`,data);
}

export const leaveServer=async(serverId:string)=>{
    return apiClient.delete(`/servers/${serverId}/leave`);
}

export const deleteServer=async(serverId:string)=>{
    return apiClient.delete(`/servers/${serverId}`);
}

export const kickMember = async (data:KickMemberInput) => {
    return apiClient.delete(`/servers/${data.serverId}/members/${data.memberId}/kick`);
};


