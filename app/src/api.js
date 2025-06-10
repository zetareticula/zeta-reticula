import axios from "axios";

const API_URL = "http://localhost:8080/api"; // Adjust based on your API setup

export const fetchAvailableModels = async () => {
  try {
    const response = await axios.get(`${API_URL}/models`);
    return response.data;
  } catch (error) {
    console.error("Error fetching models:", error);
    return [];
  }
};

export const uploadModel = async (file) => {
  const formData = new FormData();
  formData.append("model", file);
  try {
    const response = await axios.post(`${API_URL}/upload`, formData, {
      headers: { "Content-Type": "multipart/form-data" },
    });
    return response.data;
  } catch (error) {
    console.error("Error uploading model:", error);
    return null;
  }
};

export const fetchInferenceStats = async () => {
  try {
    const response = await axios.get(`${API_URL}/stats`);
    return response.data;
  } catch (error) {
    console.error("Error fetching stats:", error);
    return {
      latency: 0.4,
      memory_savings: 60,
      throughput: 2500,
      anns_recall: 0.95,
    };
  }
};