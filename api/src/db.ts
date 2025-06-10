import { Pool } from '@neondatabase/serverless';

const pool = new Pool({
  connectionString: process.env.DATABASE_URL, // Set in Vercel env vars
});

export const query = async (text: string, params: any[] = []) => {
  const client = await pool.connect();
  try {
    const result = await client.query(text, params);
    return result.rows;
  } finally {
    client.release();
  }
};

// Initialize tables
export const initDB = async () => {
  await query(`
    CREATE TABLE IF NOT EXISTS models (
      model_id VARCHAR(36) PRIMARY KEY,
      user_id VARCHAR(50) NOT NULL,
      file_name VARCHAR(255) NOT NULL,
      file_size BIGINT NOT NULL,
      format VARCHAR(50) NOT NULL,
      quantized_path TEXT NOT NULL,
      upload_timestamp TIMESTAMP NOT NULL,
      cost DECIMAL(10, 4) NOT NULL
    );
  `);
  await query(`
    CREATE TABLE IF NOT EXISTS usage (
      id SERIAL PRIMARY KEY,
      user_id VARCHAR(50) NOT NULL,
      tokens_used INTEGER NOT NULL,
      cost DECIMAL(10, 4) NOT NULL,
      timestamp TIMESTAMP NOT NULL
    );
  `);
};