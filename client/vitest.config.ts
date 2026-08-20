import { defineConfig } from "vitest/config";

// The frontend's pure logic — session labels, grouping, aging — is arithmetic
// that cannot be checked by looking at the app, so it gets a test runner. The
// React half is still verified by running it (T-302's note); this is not a
// component-testing setup.
export default defineConfig({
  test: {
    include: ["src/**/*.test.ts"],
    environment: "node",
  },
});
