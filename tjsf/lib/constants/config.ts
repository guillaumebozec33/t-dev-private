export const config = {
  apiUrl: process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001/api',
  wsUrl: process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:3001',
} as const;
