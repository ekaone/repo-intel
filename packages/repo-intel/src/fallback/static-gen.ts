/**
 * Skill name → human-readable description for static doc generation.
 * Extend this map to add descriptions for more skills.
 */
export const SKILL_DESCRIPTIONS: Record<string, string> = {
  // Languages
  typescript: "Write fully-typed TypeScript, leverage strict mode, and use type inference over explicit annotations where possible",
  rust: "Use Rust idioms: ownership, borrowing, error propagation with `?`, and async/await with Tokio",
  nodejs: "Target Node.js ESM modules, async/await patterns, and built-in test runner where appropriate",
  python: "Follow PEP 8, type-hint all functions, and use async patterns via asyncio where needed",
  go: "Follow Go conventions: error wrapping, interfaces for dependency injection, and stdlib-first approach",

  // Frameworks
  nextjs: "Use Next.js App Router conventions: server components by default, minimal client components, route handlers for APIs",
  react: "Prefer functional components, hooks over class components, and keep state as close to usage as possible",
  vue: "Use the Composition API with `<script setup>`, and keep reactive state in composables",
  angular: "Follow Angular module/component/service patterns and use RxJS for async state",
  axum: "Structure handlers as async functions with typed extractors; use layers for middleware",
  actix: "Use `actix-web` extractors and scope-based routing; handle errors via `ResponseError`",
  express: "Keep middleware lean; prefer typed request/response with custom type augmentation",
  fastify: "Use Fastify schema validation and typed requests with `zod` or JSON Schema",
  nestjs: "Follow NestJS module/controller/service DI patterns; use pipes for validation",

  // Tooling
  prisma: "Use Prisma Client with generated types; never write raw SQL unless confirmed necessary",
  drizzle: "Use Drizzle ORM schema definitions and typed query builder",
  vite: "Configure Vite plugins in `vite.config.ts`; leverage HMR for fast iteration",
  vitest: "Structure tests with `describe`/`it`/`expect`; use `vi.mock` for module mocking",
  jest: "Write tests with `describe`/`it`/`expect`; mock modules with `jest.mock`",
  turborepo: "Respect Turborepo pipeline dependencies in `turbo.json`; use `--filter` for targeted builds",
  docker: "Keep Docker images minimal; use multi-stage builds to separate build and runtime layers",
  biome: "Run `biome check` before committing; prefer auto-fixable rules over suppressions",
};
