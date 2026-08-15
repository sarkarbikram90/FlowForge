/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        background: '#0B0E17',
        surface: {
          DEFAULT: '#111827',
          50: '#1F293D',
          100: '#172033',
          200: '#121A2B',
          300: '#0E1422',
        },
        border: '#1E293B',
      },
    },
  },
  plugins: [],
}
