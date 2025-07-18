import React from 'react';

const solutions = [
  { title: 'Smart Processing', desc: 'Speed up your AI with efficient tools.' },
  { title: 'Scalable Insights', desc: 'Grow with solutions that adapt to your needs.' },
  { title: 'Trusted Accuracy', desc: 'Rely on precise results every time.' },
];

const Solutions: React.FC = () => {
  return (
    <section className="py-16">
      <div className="container mx-auto px-4">
        <h2 className="text-3xl font-bold text-center mb-8">What We Offer</h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
          {solutions.map((solution, index) => (
            <div key={index} className="bg-white p-6 rounded-lg shadow-lg text-center">
              <h3 className="text-xl font-semibold mb-2">{solution.title}</h3>
              <p className="text-gray-600">{solution.desc}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
};

export default Solutions;