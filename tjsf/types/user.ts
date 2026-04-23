import { z } from 'zod';

export const StoredUserStatusSchema = z.enum(['online', 'away', 'donotdisturb','dnd', 'invisible']);
export type StoredUserStatus = z.infer<typeof StoredUserStatusSchema>;

export const UserStatusSchema = z.enum(['online', 'away', 'donotdisturb','dnd', 'invisible', 'offline']);
export type UserStatus = z.infer<typeof UserStatusSchema>;

export const UserSchema = z.object({
  id: z.string(),
  username: z.string().min(1).max(50),
  password: z.string(),
  email: z.string(),
  avatar_url: z.string().url().nullable(),
  status: UserStatusSchema,
});


export type User = z.infer<typeof UserSchema>;

export const LoginSchema = UserSchema.partial().required({email:true,password:true});
export type Login = z.infer<typeof LoginSchema>;

export const RegisterSchema = UserSchema.partial().required({email:true,password: true, username:true});
export type Register = z.infer<typeof RegisterSchema>;
