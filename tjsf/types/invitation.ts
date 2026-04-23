import { z } from 'zod';

export const CreateInvitationSchema = z.object({
  server_id: z.string(),
  max_uses: z.number()
});

export type CreateInvitationInput = z.infer<typeof CreateInvitationSchema>;

export const InvitationOutputSchema = z.object({
    code:z.string(),
    server_id:z.string(),
    max_uses:z.number(),
    uses:z.number(),
    expires_at:z.string().datetime().nullable()
});

export type InvitationOutput = z.infer<typeof InvitationOutputSchema>;

export const InvitationSchema = z.object({
    invite_code:z.string()
})
export type Invitation =  z.infer<typeof InvitationSchema>;