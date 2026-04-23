import { z } from 'zod';

export const MessageSchema = z.object({
  id: z.string(),
  author_id: z.string(),
  channel_id: z.string(),
  content: z.string().min(1).max(2000),
  author_username : z.string(),
  author_avatar_url : z.string(),
  edited : z.boolean(),
  created_at: z.string().datetime(),
  updated_at: z.string().datetime().nullable()
});

export type Message = z.infer<typeof MessageSchema>;

// Envoyer un message
export const SendMessageSchema = MessageSchema.partial().required({channel_id: true,content:true})

export type SendMessage = z.infer<typeof SendMessageSchema>;
