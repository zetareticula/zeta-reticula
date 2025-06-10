import React, { useState, useEffect } from "react";
import { fetchAvailableModels } from "../api";

const ModelSelector = ({ onSelect }) => {
  const [models, setModels] = useState([]);
  const [selectedModel, setSelectedModel] = useState("");

  useEffect(() => {
    const loadModels = async () => {
      const availableModels = await fetchAvailableModels();
      setModels(availableModels);
      if (availableModels.length > 0) {
        setSelectedModel(availableModels[0]);
        onSelect(availableModels[0]);
      }
    };
    loadModels();
  }, [onSelect]);

  const handleChange = (e) => {
    const model = e.target.value;
    setSelectedModel(model);
    onSelect(model);
  };

  return (
    <div className="bg-cosmic-light p-6 rounded-lg shadow-glow">
      <h2 className="text-lg font-bold text-cosmic-glow mb-4">
        Select an Open LLM Model
      </h2>
      <select
        value={selectedModel}
        onChange={handleChange}
        className="w-full p-2 rounded bg-cosmic-dark text-cosmic-glow border border-cosmic-accent focus:outline-none"
      >
        {models.map((model) => (
          <option key={model} value={model}>
            {model}
          </option>
        ))}
      </select>
    </div>
  );
};

export default ModelSelector;