import { z } from 'zod';

export const ChannelSchema = z.object({
  id: z.string(),
  name: z.string().min(1).max(50),
  description: z.string().max(500).nullable(),
  server_id: z.string(),
  channel_type: z.string(),
  position: z.number(),
  is_private: z.boolean(),
  icon: z.string().nullable().optional(),
  created_at:z.string().datetime(),
});

export type Channel = z.infer<typeof ChannelSchema>;

export const EditChannelSchema = ChannelSchema.partial().required({ id: true,name:true });
export type EditChannel = z.infer<typeof EditChannelSchema>;

export const CreateChannelSchema = ChannelSchema.partial().required({name:true,server_id:true});
export type CreateChannel = z.infer<typeof CreateChannelSchema>;