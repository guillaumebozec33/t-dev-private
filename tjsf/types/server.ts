import { z } from 'zod';

// Server
export const ServerSchema = z.object({
  id: z.string(),
  name: z.string().min(1).max(100),
  description: z.string().max(500).nullable(),
  owner_id: z.string(),
  icon_url:z.url().nullable(),
  created_at:z.string().datetime(),
});

export type Server = z.infer<typeof ServerSchema>;

export const CreateServerSchema = ServerSchema.partial().required({ name: true, description: true });
export type CreateServer = z.infer<typeof CreateServerSchema>;

export const EditServerSchema = ServerSchema.partial().required({id:true,name:true,description: true});
export type EditServer = z.infer<typeof EditServerSchema>;

export const KickMemberInputSchema = z.object({
    serverId: z.string(),
    memberId: z.string()
})
export type KickMemberInput = z.infer<typeof KickMemberInputSchema>;