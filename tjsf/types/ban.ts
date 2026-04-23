import { z } from 'zod';

export const BanSchema = z.object({
  id: z.string(),
  user_id: z.string(),
  server_id: z.string(),
  banned_by: z.string(),
    username:z.string(),
  banned_at: z.string().datetime(),
  expires_at: z.string().datetime(),
  is_permanent: z.boolean(),
});

export type Ban = z.infer<typeof BanSchema>;

export const BanMemberInputSchema = z.object({
  serverId: z.string(),
  memberId: z.string(),
  duration_hours: z.number().optional(), // undefined = permanent
});

export type BanMemberInput = z.infer<typeof BanMemberInputSchema>;

export const UnbanMemberInputSchema = z.object({
  serverId: z.string(),
  userId: z.string(),
});

export type UnbanMemberInput = z.infer<typeof UnbanMemberInputSchema>;
