import { drizzle } from 'drizzle-orm/neon-serverless';
import { neon } from '@neon-js';
import * as schema from './schema';

const sql = neon(process.env.NEON_CONNECTION_STRING!);
export const db = drizzle(sql, { schema });

// Example schema (adjust based on needs)
export const users = schema.pgTable('users', {
  id: schema.serial('id').primaryKey(),
  name: schema.text('name'),
  email: schema.text('email').unique(),
});

export const feedback = schema.pgTable('feedback', {
  id: schema.serial('id').primaryKey(),
  userId: schema.integer('user_id').references(() => users.id),
  message: schema.text('message'),
  createdAt: schema.timestamp('created_at').defaultNow(),
});