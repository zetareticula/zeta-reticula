import React from 'react';

const Hero: React.FC = () => {
  return (
    <section className="bg-primary text-white py-20">
      <div className="container mx-auto text-center px-4">
        <h1 className="text-5xl font-bold mb-4">Zeta Reticula AI</h1>
        <p className="text-xl mb-6">Revolutionize inference with cutting-edge quantization technology.</p>
        <button className="bg-accent text-white px-6 py-3 rounded-lg hover:bg-yellow-600 transition">
          Get Started
        </button>
      </div>
    </section>
  );
};

export default Hero;