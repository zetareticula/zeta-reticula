/** @type {import('tailwindcss').Config} */
module.exports = {
    content: [
      './src/**/*.{js,ts,jsx,tsx}',
    ],
    theme: {
      extend: {
        colors: {
          primary: '#1E3A8A', // Corporate blue
          secondary: '#6B46C1', // Purple accent
          accent: '#FBBF24',   // Warm yellow
        },
      },
    },
    plugins: [],
  };