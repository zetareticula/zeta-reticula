import React from "react";

const Navbar = () => {
  return (
    <nav className="bg-cosmic-dark shadow-glow p-4">
      <div className="container mx-auto flex justify-between items-center">
        <h1 className="text-2xl font-bold text-cosmic-glow">
          Zeta Reticula 🌌
        </h1>
        <ul className="flex space-x-4">
          <li>
            <a href="#" className="text-cosmic-light hover:text-cosmic-accent">
              Dashboard
            </a>
          </li>
          <li>
            <a href="#" className="text-cosmic-light hover:text-cosmic-accent">
              Models
            </a>
          </li>
          <li>
            <a href="#" className="text-cosmic-light hover:text-cosmic-accent">
              Settings
            </a>
          </li>
        </ul>
      </div>
    </nav>
  );
};

export default Navbar;