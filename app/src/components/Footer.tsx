import React from 'react';

const Footer: React.FC = () => {
  return (
    <footer className="bg-gray-800 text-white py-6">
      <div className="container mx-auto text-center px-4">
        <p>&copy; {new Date().getFullYear()} Zeta Reticula AI. All rights reserved.</p>
        <div className="mt-2">
          <a href="#" className="text-accent hover:underline mx-2">Privacy</a>
          <a href="#" className="text-accent hover:underline mx-2">Terms</a>
        </div>
      </div>
    </footer>
  );
};

export default Footer;