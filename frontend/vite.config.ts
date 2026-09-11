/// <reference types="vitest/config" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// Tailwind v4 Vite integration: the @tailwindcss/vite plugin + CSS-first
// @import "tailwindcss" — no postcss config, no v3 tailwind.config
// (knowledge frontend-stack.md).
export default defineConfig({
  plugins: [react(), tailwindcss()],
  test: {
    environment: "jsdom",
  },
});
