/** @type {import('tailwindcss').Config} */
module.exports = {
    content: ["./src/**/*.{js,jsx,ts,tsx}"],
    theme: {
      extend: {
        colors: {
          cosmic: {
            dark: "#1a1b2e",
            light: "#2e335a",
            accent: "#6b5be3",
            glow: "#a29bfe",
          },
        },
        fontFamily: {
          orbit: ["Orbitron", "sans-serif"],
          mono: ["Fira Code", "monospace"],
        },
        backgroundImage: {
          "cosmic-gradient": "linear-gradient(135deg, #1a1b2e 0%, #2e335a 100%)",
        },
        boxShadow: {
          "glow": "0 0 15px rgba(162, 155, 254, 0.5)",
        },
      },
    },
    plugins: [],
  };