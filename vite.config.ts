import { defineConfig } from "vite-plus"

const ignorePatterns = ["**/generated/**", "rust/target/**"]

export default defineConfig({
  fmt: {
    semi: false,
    printWidth: 90,
    arrowParens: "avoid",
    // The layout is a minijinja template, not HTML a formatter can reflow safely
    ignorePatterns: [...ignorePatterns, "**/CHANGELOG.md", "rust/src/layout.html"],
    // Docs code snippets may run to 100 columns, the width the docs guard allows
    overrides: [{ files: ["**/*.md"], options: { printWidth: 100 } }],
  },
  lint: {
    ignorePatterns,
    options: {
      typeAware: false,
      typeCheck: false,
    },
  },
  test: {
    include: [".scripts/-test/*.unit.ts"],
    env: { NODE_OPTIONS: "--no-warnings" },
    testTimeout: 60 * 1000,
    silent: "passed-only",
  },
})
