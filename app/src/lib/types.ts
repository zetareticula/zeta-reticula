export interface User {
    id: number;
    name: string;
    email: string;
  }
  
  export interface Feedback {
    id: number;
    userId: number;
    message: string;
    createdAt: Date;
  }

export interface Subscription {
    id: number;
    userId: number;
    plan: string;
    status: string;
    startDate: Date;
    endDate?: Date | null;
}

export interface Model {
    modelId: string;
    userId: number;
    fileName: string;
    fileSize: number;
    format: string;
    quantizedPath: string;
    uploadTimestamp: Date;
    cost: number;
}

export interface Usage {
    id: number;
    userId: number;
    tokensUsed: number;
    cost: number;
    timestamp: Date;
}

export interface ModelUpload {
    file: File;
    userId: number;
    format: string;
}

export interface ModelDownload {
    modelId: string;
    userId: number;
}
export interface ModelQuantization {
    modelId: string;
    userId: number;
    targetFormat: string;
}

export interface ModelDeletion {
    modelId: string;
    userId: number;
}

export interface ModelList {
    userId: number;
}

export interface ModelSearch {
    query: string;
    userId: number;
}
