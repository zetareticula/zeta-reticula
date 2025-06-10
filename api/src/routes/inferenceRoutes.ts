import { Router, Request, Response } from 'express';
import { postInference, getUsageHistory } from '../controllers/inferenceController';
import { authenticate } from '../middleware/authMiddleware';

// Export as a Vercel serverless function
export const handler = async (req: Request, res: Response) => {
  const router = Router();
  router.post('/', authenticate, postInference);
  router.get('/usage', authenticate, getUsageHistory);

  // Vercel expects a handler function
  return new Promise<void>((resolve) => {
    router(req, res, () => resolve());
  });
};

export default handler;