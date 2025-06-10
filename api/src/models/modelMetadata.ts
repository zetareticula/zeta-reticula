import { query } from '../db';

interface ModelMetadata {
  model_id: string;
  user_id: string;
  file_name: string;
  file_size: number;
  format: string;
  quantized_path: string;
  upload_timestamp: string;
  cost: number;
}

export const addModel = async (metadata: ModelMetadata) => {
  await query(
    `INSERT INTO models (model_id, user_id, file_name, file_size, format, quantized_path, upload_timestamp, cost)
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)`,
    [
      metadata.model_id,
      metadata.user_id,
      metadata.file_name,
      metadata.file_size,
      metadata.format,
      metadata.quantized_path,
      metadata.upload_timestamp,
      metadata.cost,
    ]
  );
};

export const getModel = async (modelId: string): Promise<ModelMetadata | undefined> => {
  const rows = await query('SELECT * FROM models WHERE model_id = $1', [modelId]);
  return rows[0] as ModelMetadata | undefined;
};