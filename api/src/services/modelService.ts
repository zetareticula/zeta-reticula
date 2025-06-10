import { saveFile, validateModelFile } from '../utils/fileUtils';
import { calculateUploadCost } from '../utils/billing';
import { addModel } from '../models/modelMetadata';
import { addUsage } from '../models/userUsage';
import { v4 as uuidv4 } from 'uuid';

export const uploadModel = async (file: Express.Multer.File, userId: string): Promise<{ modelId: string; cost: number }> => {
  const error = validateModelFile(file);
  if (error) throw new Error(error);

  const fileUrl = await saveFile(file);
  const fileSize = file.size;
  const cost = calculateUploadCost(fileSize);

  const modelId = uuidv4();
  const metadata: ModelMetadata = {
    model_id: modelId,
    user_id: userId,
    file_name: file.originalname,
    file_size: fileSize,
    format: path.extname(file.originalname).toLowerCase(),
    quantized_path: `${fileUrl}.rkv`, // Simulate quantization
    upload_timestamp: new Date().toISOString(),
    cost,
  };

  console.log(`Quantizing ${file.originalname} to ${metadata.quantized_path}`);
  
  await addModel(metadata);
  await addUsage({
    user_id: userId,
    tokens_used: 0,
    cost,
    timestamp: metadata.upload_timestamp,
  });

  return { modelId, cost };
};