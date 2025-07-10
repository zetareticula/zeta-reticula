// Placeholder implementations for saveFile and validateModelFile
import type { Express } from 'express';

export async function saveFile(file: Express.Multer.File): Promise<string> {
  // Return a dummy file URL
  return `/mock/path/${file.originalname}`;
}

export function validateModelFile(file: Express.Multer.File): string | null {
  // Always return null (no error)
  return null;
}
