import { z } from 'zod';
import {UserSchema} from "@/types/user";

export const AuthOutputSchema = z.object({
    token: z.string(),
    user:UserSchema
});

export type AuthOutput = z.infer<typeof AuthOutputSchema>;

