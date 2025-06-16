import React from 'react';

const CTA: React.FC = () => {
  return (
    <section className="bg-secondary text-white py-12">
      <div className="container mx-auto text-center px-4">
        <h2 className="text-3xl font-bold mb-4">Ready to Optimize Your AI?</h2>
        <p className="mb-6">Join the future of inference quantization today.</p>
        <button className="bg-accent text-white px-6 py-3 rounded-lg hover:bg-yellow-600 transition">
          Contact Us
        </button>
      </div>
    </section>
  );
};

export default CTA;