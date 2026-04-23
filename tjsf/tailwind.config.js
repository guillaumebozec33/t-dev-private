/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './app/**/*.{js,ts,jsx,tsx,mdx}',
    './components/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  theme: {
    extend: {
      colors: {
        discord: {
          dark: 'var(--discord-dark)',
          darker: 'var(--discord-darker)',
          darkest: 'var(--discord-darkest)',
          sidebar: 'var(--discord-sidebar)',
          bg: 'var(--discord-bg)',
          hover: 'var(--discord-hover)',
          accent: 'var(--discord-accent)',
          'accent-hover': 'var(--discord-accent-hover)',
          text: 'var(--discord-text)',
          'text-muted': 'var(--discord-text-muted)',
          'text-link': 'var(--discord-text-link)',
          error: 'var(--discord-error)',
          warning: 'var(--discord-warning)',
          success: 'var(--discord-success)',
          border: 'var(--discord-border)',
        },
      },
    },
  },
  plugins: [],
}
