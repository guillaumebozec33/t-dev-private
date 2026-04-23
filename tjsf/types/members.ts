import {z} from "zod";
import { UserStatusSchema } from './user';

export const UserServerSchema = z.object({
  id:z.string(),
  user_id: z.string(),
    username:z.string(),
  server_id: z.string(),
  role: z.string(),
  status: UserStatusSchema.optional(),
  joined_at: z.string().datetime(),
  avatar_url: z.string().nullable().optional(),
});

export type Member = z.infer<typeof UserServerSchema>;

export const ChangeRoleSchema = UserServerSchema.partial().required({user_id:true,server_id:true,role:true});
export type ChangeRole = z.infer<typeof ChangeRoleSchema>;

export const TransferOwnershipInputSchema = z.object({
    new_owner_id:z.string(),
    server_id:z.string()
})

export type TransferOwnershipInput = z.infer<typeof TransferOwnershipInputSchema>;