import type { Config } from "tailwindcss";

const config: Config = {
  content: [
    "./pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./components/**/*.{js,ts,jsx,tsx,mdx}",
    "./app/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        background: "var(--background)",
        foreground: "var(--foreground)",
        bordeaux: {
          DEFAULT: "#C1121F",
          hover: "#9D0208",
          dark: "#6A040F",
          light: "#F8E8E9",
        },
        gold: {
          DEFAULT: "#FFB703",
          dark: "#FB8500",
          light: "#FFF8E7",
        },
        "sidebar-bg": "#F8F9FA",
        "sidebar-hover": "#E9ECEF",
        "steel-blue": {
          DEFAULT: "#023E8A",
          dark: "#03045E",
          light: "#E7F0FF",
        },
        danger: {
          DEFAULT: "#DC2626",
          hover: "#B91C1C",
          light: "#FEE2E2",
        },
        warning: {
          DEFAULT: "#F59E0B",
          hover: "#D97706",
          light: "#FEF3C7",
        },
        success: {
          DEFAULT: "#10B981",
          hover: "#059669",
          light: "#D1FAE5",
        },
      },
    },
  },
  plugins: [],
};
export default config;
