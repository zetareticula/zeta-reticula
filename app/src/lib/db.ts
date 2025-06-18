import { drizzle } from 'drizzle-orm/neon-serverless';
import { neon } from 'drizzle-orm/neon-serverless';
import * as schema from './schema'; // Import your schema definitions
import 'dotenv/config';
import { config } from 'dotenv';
// Load environment variables from .env file
config();

// Ensure the dotenv package is configured to load environment variables
// Check if the environment variable is set
if (!process.env.DATABASE_URL) {
  throw new Error('DATABASE_URL environment variable is not set');
}


// Ensure the environment variable is set
if (!process.env.NEON_CONNECTION_STRING) {
  throw new Error('NEON_CONNECTION_STRING environment variable is not set');
}

// Initialize the Neon connection using the connection string from environment variables
const sql = neon(process.env.NEON_CONNECTION_STRING!); // Ensure the connection string is not null
export const db = drizzle(sql, { schema }); // Export the database instance for use in your application

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

export const usage = schema.pgTable('usage', {
    id: schema.serial('id').primaryKey(),
    userId: schema.integer('user_id').references(() => users.id),
    tokensUsed: schema.integer('tokens_used'),
    cost: schema.numeric('cost', { precision: 10, scale: 4 }),
    timestamp: schema.timestamp('timestamp').defaultNow(),
});

// Export the schema for use in migrations or other parts of the application
export { schema };

// Note: Ensure that the Neon connection string is correctly set in your environment variables.
// This code initializes a Drizzle ORM instance with Neon serverless, defines the database schema, and exports the database instance for use in your application.
// Adjust the schema definitions based on your actual database structure and requirements.

export const modelUploads = schema.pgTable('model_uploads', {
    file: schema.blob('file'),
    userId: schema.integer('user_id').references(() => users.id),
    format: schema.text('format'),
});

export const modelDownloads = schema.pgTable('model_downloads', {
    modelId: schema.text('model_id').references(() => models.modelId),
    userId: schema.integer('user_id').references(() => users.id),
});

export const modelQuantizations = schema.pgTable('model_quantizations', {
    modelId: schema.text('model_id').references(() => models.modelId),
    userId: schema.integer('user_id').references(() => users.id),
    targetFormat: schema.text('target_format'),
});

export const modelDeletions = schema.pgTable('model_deletions', {
    modelId: schema.text('model_id').references(() => models.modelId),
    userId: schema.integer('user_id').references(() => users.id),
});

// Ensure that the schema is correctly defined and matches your database structure.