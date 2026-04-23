import { apiClient } from '@/lib/api/client';
import {User, StoredUserStatus} from '@/types';

interface UpdateMyProfilePayload {
    username?: string;
    avatar_url?: string|null;
    status?: StoredUserStatus;
}

export const getMe = async()=>{
    return apiClient.get<User>('/me');
}

export const updateMyStatus = async(status: StoredUserStatus)=>{
    return apiClient.patch<User>('/me', { status });
}

export const updateMyProfile = async(payload: UpdateMyProfilePayload)=>{
    return apiClient.patch<User>('/me', payload);
}
