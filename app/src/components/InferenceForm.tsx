'use client';

import React, { useState } from 'react';
import { runInference } from '../lib/api';

const InferenceForm: React.FC = () => {
  const [input, setInput] = useState('');
  const [options, setOptions] = useState({ dynamic: false, federated: false });
  const [modelId, setModelId] = useState('');
  const [result, setResult] = useState<{ output: string; tokensUsed: number; cost: number } | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    try {
      const response = await runInference({ input, options, modelId });
      setResult(response);
    } catch (err) {
      setError('Inference failed. Please try again.');
    }
  };

  return (
    <div>
      <form onSubmit={handleSubmit}>
        <input
          type="text"
          value={modelId}
          onChange={(e) => setModelId(e.target.value)}
          placeholder="Enter Model ID (optional)"
          style={{ width: '100%', marginBottom: '10px' }}
        />
        <textarea
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Enter text for inference..."
          rows={5}
          style={{ width: '100%' }}
        />
        <div>
          <label>
            <input
              type="checkbox"
              checked={options.dynamic}
              onChange={() => setOptions({ ...options, dynamic: !options.dynamic })}
            />
            Dynamic Inference (+$0.0005/1K tokens)
          </label>
        </div>
        <div>
          <label>
            <input
              type="checkbox"
              checked={options.federated}
              onChange={() => setOptions({ ...options, federated: !options.federated })}
            />
            Federated Inference (+$0.002/1K tokens)
          </label>
        </div>
        <button type="submit">Run Inference</button>
      </form>
      {result && (
        <div>
          <h3>Result</h3>
          <p><strong>Output:</strong> {result.output}</p>
          <p><strong>Tokens Used:</strong> {result.tokensUsed}</p>
          <p><strong>Cost:</strong> ${result.cost.toFixed(4)}</p>
        </div>
      )}
      {error && <p style={{ color: 'red' }}>{error}</p>}
    </div>
  );
};

export default InferenceForm;