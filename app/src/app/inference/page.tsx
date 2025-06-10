'use client';

import InferenceForm from '../../components/InferenceForm';
import UsageHistory from '../../components/UsageHistory';

export default function InferencePage() {
  return (
    <div className="container">
      <h2>Run Inference</h2>
      <InferenceForm />
      <UsageHistory userId="client-123" /> {/* Hardcoded for prototype */}
    </div>
  );
}