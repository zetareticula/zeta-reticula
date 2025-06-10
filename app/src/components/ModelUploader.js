import React, { useState } from "react";
import { uploadModel } from "../api";

const ModelUploader = ({ onUpload }) => {
  const [file, setFile] = useState(null);
  const [status, setStatus] = useState("");

  const handleFileChange = (e) => {
    setFile(e.target.files[0]);
  };

  const handleUpload = async () => {
    if (!file) {
      setStatus("Please select a file to upload.");
      return;
    }
    setStatus("Uploading...");
    const result = await uploadModel(file);
    if (result) {
      setStatus("Model uploaded successfully!");
      onUpload(result.modelName);
    } else {
      setStatus("Upload failed. Please try again.");
    }
  };

  return (
    <div className="bg-cosmic-light p-6 rounded-lg shadow-glow">
      <h2 className="text-lg font-bold text-cosmic-glow mb-4">
        Upload Your Own Model
      </h2>
      <input
        type="file"
        onChange={handleFileChange}
        className="mb-4 text-cosmic-glow"
      />
      <button
        onClick={handleUpload}
        className="bg-cosmic-accent text-white px-4 py-2 rounded hover:bg-cosmic-glow transition"
      >
        Upload Model
      </button>
      {status && <p className="mt-2 text-cosmic-glow">{status}</p>}
    </div>
  );
};

export default ModelUploader;