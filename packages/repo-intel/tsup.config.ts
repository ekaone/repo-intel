import { defineConfig } from "tsup";

export default defineConfig([
  // ── Library (SDK) entry ────────────────────────────────────────────────────
  // No shebang — this is imported by SDK consumers as a normal module.
  {
    entry: { index: "src/index.ts" },
    format: ["esm", "cjs"],
    dts: true,
    clean: true,
    sourcemap: true,
    shims: true,
    outDir: "dist",
    target: "node18",
  },

  // ── CLI entry ──────────────────────────────────────────────────────────────
  // Built as CJS to avoid the shebang-in-ESM SyntaxError on Node 24 + Windows.
  // When "type": "module" is set, Node treats .js as ESM and rejects the shebang
  // as an invalid token. CJS (.cjs) doesn't have this restriction.
  {
    entry: { cli: "src/cli.ts" },
    format: ["cjs"],
    outExtension: () => ({ js: ".cjs" }),
    dts: false,
    clean: false,
    sourcemap: true,
    shims: true,
    banner: {},
    outDir: "dist",
    target: "node18",
  },
]);
