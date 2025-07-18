import React from 'react';

const About: React.FC = () => {
  return (
    <section className="py-16 bg-gray-100">
      <div className="container mx-auto px-4 text-center">
        <h2 className="text-3xl font-bold mb-6">About Us</h2>
        <p className="text-lg text-gray-700 max-w-2xl mx-auto">
          At Zeta Reticula, we’re passionate about making AI work smarter for everyone. Our team
          combines expertise with cutting-edge tools to deliver solutions that save time and
          enhance decision-making.
        </p>
      </div>
    </section>
  );
};

export default About;