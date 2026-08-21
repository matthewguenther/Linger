import { defineConfig } from "vitest/config";

// The frontend's pure logic — session labels, grouping, aging, the markdown
// parser — is arithmetic that cannot be checked by looking at the app, so it
// gets a test runner.
//
// This is still not a component-testing setup: there is no DOM here and no
// testing library. The one `.tsx` file in it renders `Markdown` to a *string*
// with `react-dom/server` and reads the string, because T-304's accept
// criterion is about the markup a browser would be handed, and asserting on the
// markup itself is the only way to prove it. The rest of the React half is
// verified by running the app.
export default defineConfig({
  test: {
    include: ["src/**/*.test.ts", "src/**/*.test.tsx"],
    environment: "node",
  },
});
