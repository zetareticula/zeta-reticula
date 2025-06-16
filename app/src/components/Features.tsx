import React from 'react';

const features = [
  { title: 'Efficient Quantization', desc: 'Leverage KIVI and BinaryMoS for optimal performance.' },
  { title: 'Scalable Models', desc: 'Support for large-scale inference with LoRA.' },
  { title: 'Knowledge Distillation', desc: 'Enhance accuracy with QAKD from full-precision models.' },
];

const Features: React.FC = () => {
  return (
    <section className="py-16 bg-gray-100">
      <div className="container mx-auto px-4">
        <h2 className="text-3xl font-bold text-center mb-8">Our Features</h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
          {features.map((feature, index) => (
            <div key={index} className="bg-white p-6 rounded-lg shadow-lg">
              <h3 className="text-xl font-semibold mb-2">{feature.title}</h3>
              <p className="text-gray-600">{feature.desc}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
};

export default Features;