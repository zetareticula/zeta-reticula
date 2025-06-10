'use client';

import React, { useState } from 'react';
import { uploadModel } from '../lib/api';

const ModelUpload: React.FC = () => {
  const [file, setFile] = useState<File | null>(null);
  const [uploadStatus, setUploadStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files) setFile(e.target.files[0]);
  };

  const handleUpload = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!file) {
      setError('Please select a file');
      return;
    }
    setError(null);
    setUploadStatus('Uploading...');
    try {
      const response = await uploadModel(file);
      setUploadStatus(`Upload successful! Model ID: ${response.modelId}, Cost: $${response.cost.toFixed(4)}`);
    } catch (err) {
      setError((err as Error).message || 'Upload failed');
    }
  };

  return (
    <div>
      <form onSubmit={handleUpload}>
        <input type="file" onChange={handleFileChange} accept=".safetensors,.onnx,.gguf" />
        <button type="submit">Upload</button>
      </form>
      {uploadStatus && <p>{uploadStatus}</p>}
      {error && <p style={{ color: 'red' }}>{error}</p>}
    </div>
  );
};

export default ModelUpload;