import {Register,Login} from '@/types'
import { apiClient } from '@/lib/api/client';
import {AuthOutput} from '@/types'


export const login= async (data:Login)=>{
    return apiClient.post<AuthOutput>('/auth/login', data)
}

export const register= async(data:Register)=>{
    return apiClient.post<AuthOutput>('/auth/signup', data)
}
