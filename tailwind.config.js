/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      boxShadow: {
        glass:
          'inset 0 1px 0 0 rgb(255 255 255 / 0.05), 0 8px 24px -12px rgb(0 0 0 / 0.7)',
        'glow-amber': '0 0 20px -6px rgb(245 158 11 / 0.3)',
        'glow-emerald': '0 0 20px -6px rgb(16 185 129 / 0.3)',
        'glow-red': '0 0 20px -6px rgb(239 68 68 / 0.3)',
      },
      transitionTimingFunction: {
        'out-quart': 'cubic-bezier(0.25, 1, 0.5, 1)',
        spring: 'cubic-bezier(0.34, 1.56, 0.64, 1)',
      },
      transitionDuration: {
        400: '400ms',
      },
      keyframes: {
        'fade-rise': {
          from: { opacity: '0', transform: 'translateY(8px)' },
          to: { opacity: '1', transform: 'none' },
        },
        'scale-in': {
          from: { opacity: '0', transform: 'scale(0.96)' },
          to: { opacity: '1', transform: 'none' },
        },
        'pulse-dot': {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0.4' },
        },
      },
      animation: {
        'fade-rise': 'fade-rise 0.4s cubic-bezier(0.25, 1, 0.5, 1) both',
        'scale-in': 'scale-in 0.18s cubic-bezier(0.25, 1, 0.5, 1) both',
        'pulse-dot': 'pulse-dot 2s ease-in-out infinite',
      },
    },
  },
  plugins: [],
}
