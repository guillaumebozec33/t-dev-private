export interface Reaction {
  id: string;
  message_id: string;
  user_id: string;
  emoji: string;
  username: string | null;
}

export interface ReactionGroup {
  emoji: string;
  count: number;
  users: string[];
  hasReacted: boolean;
}
