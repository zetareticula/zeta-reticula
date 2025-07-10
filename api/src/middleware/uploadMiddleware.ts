import { Request, Response, NextFunction } from 'express';

export function uploadModelMiddleware(req: Request, res: Response, next: NextFunction) {
  // Placeholder middleware, always passes
  next();
}

export function validateUpload(req: Request, res: Response, next: NextFunction) {
  // Placeholder middleware, always passes
  next();
}
