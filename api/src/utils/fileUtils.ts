import { put } from '@vercel/blob';
import path from 'path';

export const saveFile = async (file: Express.Multer.File): Promise<string> => {
  const blob = await put(file.originalname, file.buffer, { access: 'public' });
  return blob.url;
};

export const validateModelFile = (file: Express.Multer.File): string | null => {
  const maxSize = 100 * 1024 * 1024; // 100 MB limit
  if (file.size > maxSize) return 'File size exceeds 100 MB limit';
  const allowedFormats = ['.safetensors', '.onnx', '.gguf'];
  const ext = path.extname(file.originalname).toLowerCase();
  if (!allowedFormats.includes(ext)) return 'Unsupported file format';
  return null;
};