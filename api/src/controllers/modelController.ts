import { Request, Response } from 'express';

export const postModel = (req: Request, res: Response) => {
  res.status(200).json({ message: 'Placeholder postModel' });
};

export const getModelStatus = (req: Request, res: Response) => {
  res.status(200).json({ message: 'Placeholder getModelStatus' });
};
