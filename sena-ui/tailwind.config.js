/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        sena: {
          50: '#fef7e7',
          100: '#fdecc3',
          200: '#fbdb8a',
          300: '#f9c547',
          400: '#f6ae1a',
          500: '#e6940d',
          600: '#ca7108',
          700: '#a1510b',
          800: '#844010',
          900: '#703513',
          950: '#411a06',
        },
        dark: {
          50: '#f7f7f8',
          100: '#eeeef0',
          200: '#d9d9de',
          300: '#b8b9c1',
          400: '#91939f',
          500: '#737584',
          600: '#5d5e6c',
          700: '#4c4d58',
          800: '#41424b',
          900: '#393941',
          950: '#18181b',
        },
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'Fira Code', 'monospace'],
      },
      animation: {
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'spin-slow': 'spin 3s linear infinite',
      },
    },
  },
  plugins: [],
};
