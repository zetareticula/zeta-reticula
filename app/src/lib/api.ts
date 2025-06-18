import axios from 'axios';

const API_URL = process.env.NEXT_PUBLIC_API_URL || 'https://your-vercel-domain.vercel.app/api';

interface InferenceRequest {
  input: string;
  options: { dynamic: boolean; federated: boolean };
  modelId?: string;
}

interface InferenceResponse {
  output: string;
  tokensUsed: number;
  cost: number;
}

interface Usage {
  user_id: string;
  tokens_used: number;
  cost: number;
  timestamp: string;
}

interface ModelUploadResponse {
  modelId: string;
  cost: number;
  message: string;
}


// Define the types for the API responses
// and requests based on your application's needs
// Example types for inference, usage, and model upload
// Adjust these interfaces based on your actual API response structure
// and request payloads
// Ensure you have the correct types for your API requests and responses

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'https://your-vercel-domain.vercel.app/api';

// Ensure the API URL is set correctly in your environment variables
if (!API_BASE_URL) {
  throw new Error('NEXT_PUBLIC_API_URL environment variable is not set');
}



const token = 'your-jwt-token'; // Replace with actual JWT token

export const runInference = async (request: InferenceRequest): Promise<InferenceResponse> => {
  const response = await axios.post<InferenceResponse>(`${API_URL}/inference`, request, {
    headers: { Authorization: `Bearer ${token}` },
  });
  return response.data;
};

export const getUsageHistory = async (userId: string): Promise<Usage[]> => {
  const response = await axios.get<Usage[]>(`/api/usage/${userId}`);
  return response.data;
};

export const uploadModel = async (file: File): Promise<ModelUploadResponse> => {
  const formData = new FormData();
  formData.append('modelFile', file);
  const response = await axios.post<ModelUploadResponse>(`${API_URL}/models`, formData, {
    headers: {
      Authorization: `Bearer ${token}`,
      'Content-Type': 'multipart/form-data',
    },
  });
  return response.data;
};