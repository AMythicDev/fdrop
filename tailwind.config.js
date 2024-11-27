import flowbitePlugin from "flowbite/plugin";

/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./src/**/*.{html,svelte}",
    "./node_modules/flowbite-svelte/**/*.{html,js,svelte,ts}",
  ],
  darkMode: "selector",
  theme: {
    extend: {},
  },
  plugins: [flowbitePlugin],
};
