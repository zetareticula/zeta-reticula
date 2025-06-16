/** @type {import('tailwindcss').Config} */
module.exports = {
    content: [
      './src/**/*.{js,ts,jsx,tsx}',
    ],
    theme: {
      extend: {
        colors: {
          primary: '#1E40AF',
          secondary: '#9333EA',
          accent: '#F59E0B',
        },
      },
    },
    plugins: [],
  };