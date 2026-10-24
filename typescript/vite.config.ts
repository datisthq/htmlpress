import { coverageConfigDefaults, defineConfig } from "vite-plus"

export default defineConfig({
  test: {
    include: ["**/*.unit.ts"],
    exclude: ["**/node_modules/**", "**/build/**", "**/compile/**"],
    env: { NODE_OPTIONS: "--no-warnings" },
    testTimeout: 60 * 1000,
    passWithNoTests: true,
    silent: "passed-only",
    coverage: {
      // Off so the run before landing stays cheap; `pnpm measure:coverage` opts in.
      enabled: false,
      reporter: ["html", "json"],
      exclude: [
        ...coverageConfigDefaults.exclude,
        "**/build/**",
        "**/compile/**",
        "**/entrypoints/**",
        "**/scripts/**",
      ],
    },
  },
})
