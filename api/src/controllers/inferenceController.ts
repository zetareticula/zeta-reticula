// Copyright 2025 ZETA RETICULA INC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import { Request, Response } from 'express';
import { InferenceService } from '../services/inferenceService';

export class InferenceController {
  private inferenceService: InferenceService;

  constructor() {
    this.inferenceService = new InferenceService();
  }

  public async runInference(req: Request, res: Response): Promise<void> {
    try {
      const result = await this.inferenceService.runInference(req.body);
      res.status(200).json(result);
    } catch (error) {
      console.error('Error running inference:', error);
      res.status(500).json({ error: 'Internal Server Error' });
    }
  }
}


