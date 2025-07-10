import { Request, Response } from 'express';

export const postInference = (req: Request, res: Response) => {
  res.status(200).json({ message: 'Placeholder postInference' });
};

export const getUsageHistory = (req: Request, res: Response) => {
  res.status(200).json({ message: 'Placeholder getUsageHistory' });
};
