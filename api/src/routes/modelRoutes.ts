import { Router, Request, Response } from 'express';
import { postModel, getModelStatus } from '../controllers/modelController';
import { authenticate } from '../middleware/authMiddleware';
import { uploadModelMiddleware, validateUpload } from '../middleware/uploadMiddleware';

export const handler = async (req: Request, res: Response) => {
  const router = Router();
  router.post('/', authenticate, uploadModelMiddleware, validateUpload, postModel);
  router.get('/:modelId', authenticate, getModelStatus);

  return new Promise<void>((resolve) => {
    router(req, res, () => resolve());
  });
};

export default handler;