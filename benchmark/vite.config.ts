import { defineConfig } from "vite-plus"

export default defineConfig({
  test: {
    include: ["-test/*.unit.ts"],
    env: { NODE_OPTIONS: "--no-warnings" },
    silent: "passed-only",
  },
})
