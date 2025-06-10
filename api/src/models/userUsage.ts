import { query } from '../db';

interface Usage {
  id?: number;
  user_id: string;
  tokens_used: number;
  cost: number;
  timestamp: string;
}

export const addUsage = async (usage: Usage) => {
  await query(
    `INSERT INTO usage (user_id, tokens_used, cost, timestamp) VALUES ($1, $2, $3, $4)`,
    [usage.user_id, usage.tokens_used, usage.cost, usage.timestamp]
  );
};

export const getUsage = async (userId: string): Promise<Usage[]> => {
  return (await query('SELECT * FROM usage WHERE user_id = $1 ORDER BY timestamp DESC', [userId])) as Usage[];
};