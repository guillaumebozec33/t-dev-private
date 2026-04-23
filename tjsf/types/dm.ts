import { z } from 'zod';

export const DmConversationSchema = z.object({
  id: z.string(),
  other_user_id: z.string(),
  other_username: z.string(),
  other_avatar_url: z.string().nullable().optional(),
  created_at: z.string(),
});

export type DmConversation = z.infer<typeof DmConversationSchema>;

export const DmMessageSchema = z.object({
  id: z.string(),
  conversation_id: z.string(),
  sender_id: z.string(),
  sender_username: z.string().nullable().optional(),
  sender_avatar_url: z.string().nullable().optional(),
  content: z.string(),
  created_at: z.string(),
});

export type DmMessage = z.infer<typeof DmMessageSchema>;
