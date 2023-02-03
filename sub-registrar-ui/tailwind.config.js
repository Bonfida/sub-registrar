const defaultTheme = require("tailwindcss/defaultTheme");

/** @type {import('tailwindcss').Config} */
module.exports = {
  presets: [require("./node_modules/@bonfida/components/tailwind.config.js")],
  content: [
    "./src/pages/**/*.{js,ts,jsx,tsx}",
    "./src/components/**/*.{js,ts,jsx,tsx}",
    "./node_modules/@bonfida/components/dist/**/*.{js,jsx,ts,tsx}",
  ],
  theme: {
    extend: {
      zIndex: { navbar: 100 },
      screens: { lg: "1315px" },
      colors: {
        "fida-gradient-end": "#201F3D",
      },
      fontFamily: {
        azeret: ["Azeret", ...defaultTheme.fontFamily.sans],
      },
    },
  },
  plugins: [require("daisyui")],
  daisyui: {
    themes: ["dark"],
  },
};
