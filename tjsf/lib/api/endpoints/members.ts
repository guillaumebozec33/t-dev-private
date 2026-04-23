import {ChangeRole,Member,TransferOwnershipInput} from "@/types";
import { apiClient } from '@/lib/api/client';

export const getMembers = async(serverId:string)=>{
    return apiClient.get<Member[]>(`/servers/${serverId}/members`);
}

export const changeRole=async(data:ChangeRole)=>{
    return apiClient.put(`/servers/${data.server_id}/members/${data.user_id}`,data)
}

export const transferOwnership=async(data:TransferOwnershipInput)=>{
    return apiClient.put(`/servers/${data.server_id}/owner`, data)
}