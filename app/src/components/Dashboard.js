import React, { useState } from "react";
import ModelSelector from "./ModelSelector";
import ModelUploader from "./ModelUploader";
import StatsPanel from "./StatsPanel";

const Dashboard = () => {
  const [selectedModel, setSelectedModel] = useState("");

  const handleModelSelect = (model) => {
    setSelectedModel(model);
  };

  const handleModelUpload = (modelName) => {
    setSelectedModel(modelName);
  };

  return (
    <div className="container mx-auto p-6">
      <h2 className="text-3xl font-bold text-cosmic-glow mb-6">
        Inference Dashboard
      </h2>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
        <ModelSelector onSelect={handleModelSelect} />
        <ModelUploader onUpload={handleModelUpload} />
      </div>
      {selectedModel && (
        <div className="mb-6">
          <p className="text-cosmic-glow">
            Selected Model: <span className="font-bold">{selectedModel}</span>
          </p>
        </div>
      )}
      <StatsPanel />
    </div>
  );
};

export default Dashboard;