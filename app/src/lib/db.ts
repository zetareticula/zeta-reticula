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

export const subscriptions = schema.pgTable('subscriptions', {
    id: schema.serial('id').primaryKey(),
    userId: schema.integer('user_id').references(() => users.id),
    plan: schema.text('plan'),
    status: schema.text('status'),
    startDate: schema.timestamp('start_date').defaultNow(),
    endDate: schema.timestamp('end_date').nullable(),
    });

export const models = schema.pgTable('models', {
    modelId: schema.text('model_id').primaryKey(),
    userId: schema.integer('user_id').references(() => users.id),
    fileName: schema.text('file_name'),
    fileSize: schema.bigint('file_size'),
    format: schema.text('format'),
    quantizedPath: schema.text('quantized_path'),
    uploadTimestamp: schema.timestamp('upload_timestamp').defaultNow(),
    cost: schema.numeric('cost', { precision: 10, scale: 4 }),
    });