/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.rs",
    "./public/index.html",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        // Obsidian Engine Design System tokens
        'bg': 'var(--color-bg, #030712)',
        'bg-secondary': 'var(--color-bg-secondary, #0d1117)',
        'bg-tertiary': 'var(--color-bg-tertiary, #161b22)',
        'bg-card': 'var(--color-bg-card, #0d1117)',
        'primary': 'var(--color-primary, #00d4ff)',
        'accent-purple': 'var(--color-accent-purple, #a855f7)',
        'accent-green': 'var(--color-accent-green, #22c55e)',
        'accent-amber': 'var(--color-accent-amber, #f59e0b)',
        'accent-rose': 'var(--color-accent-rose, #f43f5e)',
        'text-main': 'var(--color-text-main, #f8fafc)',
        'text-secondary': 'var(--color-text-secondary, #94a3b8)',
        'text-muted': 'var(--color-text-muted, #475569)',
        'border': 'var(--color-border, #1e293b)',
        'glass-bg': 'var(--color-glass-bg, rgba(13, 17, 23, 0.8))',
        'glass-border': 'var(--color-glass-border, rgba(30, 41, 59, 0.5))',
      },
      animation: {
        'float': 'float 8s ease-in-out infinite',
        'fade-in': 'fadeIn 0.3s ease-out',
        'slide-up': 'slideUp 0.3s ease-out',
        'pulse-glow': 'pulseGlow 2s ease-in-out infinite',
      },
      keyframes: {
        float: {
          '0%, 100%': { transform: 'translateY(0px) rotate(0deg)' },
          '50%': { transform: 'translateY(-20px) rotate(1deg)' },
        },
        fadeIn: {
          '0%': { opacity: '0', transform: 'translateY(5px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        slideUp: {
          '0%': { opacity: '0', transform: 'translateY(10px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        pulseGlow: {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0.7' },
        },
      },
      backdropBlur: {
        'xl': '24px',
      },
    },
  },
  plugins: [],
}
