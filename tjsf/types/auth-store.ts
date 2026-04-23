import {User} from "@/types/user";
import { z } from 'zod';

export const AuthDataSchema = z.object({
    token: z.string().nullable(),
    isAuthenticated: z.boolean(),
});

export type AuthData = z.infer<typeof AuthDataSchema>;

export interface AuthState extends AuthData {
    setToken: (token: string) => void;
    logout: () => void;
}

export interface UserState {
    user: User | null;
    selectedServerId : string | undefined,
    selectedChannelId: string | undefined,
    setSelectedServerId: (id: string | undefined) => void;
    setSelectedChannelId: (id: string | undefined) => void;
    setUser: (user: User) => void;
    clearUser: () => void;
}
