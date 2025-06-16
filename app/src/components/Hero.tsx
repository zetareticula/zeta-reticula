import React from 'react';

const Hero: React.FC = () => {
  return (
    <section className="bg-primary text-white py-20">
      <div className="container mx-auto text-center px-4">
        <h1 className="text-5xl font-bold mb-4">Welcome to Zeta Reticula</h1>
        <p className="text-xl mb-6">Unlock smarter solutions with our innovative AI technology.</p>
        <button className="bg-accent text-white px-6 py-3 rounded-lg hover:bg-yellow-500 transition">
          Explore Now
        </button>
      </div>
    </section>
  );
};

export default Hero;