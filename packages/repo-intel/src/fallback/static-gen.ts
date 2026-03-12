import type { RepoContext } from "../types";

// ── Skill descriptions ────────────────────────────────────────────────────────
// Maps a detected skill name → a one-line description used in the static doc.
// Keys are lowercase for case-insensitive lookup.

const SKILL_DESCRIPTIONS: Record<string, string> = {
  // Frameworks
  "next.js": "Server-side rendering and full-stack React framework",
  react: "Component-based UI library with a virtual DOM",
  "vue.js": "Progressive JavaScript framework for building UIs",
  svelte: "Compiled component framework with no virtual DOM overhead",
  angular: "Opinionated TypeScript framework for enterprise SPAs",
  remix: "Full-stack React framework with nested routing and loaders",
  astro: "Static site builder with partial hydration (Islands architecture)",
  "nuxt.js": "Vue-based full-stack framework with SSR and file-based routing",
  gatsby: "React static site generator with a rich plugin ecosystem",

  // Build tools
  vite: "Fast ESM-native dev server and build tool",
  webpack: "Module bundler for JavaScript applications",
  tsup: "Zero-config TypeScript bundler powered by esbuild",
  esbuild: "Extremely fast JavaScript bundler written in Go",
  turbopack: "Rust-based incremental bundler for Next.js",

  // Languages
  typescript: "Statically typed superset of JavaScript",
  rust: "Systems programming language with memory safety guarantees",
  python: "General-purpose language favoured for data science and scripting",
  go: "Compiled language optimised for concurrency and microservices",
  java: "JVM language used widely in enterprise backend systems",

  // Styling
  "tailwind css": "Utility-first CSS framework for rapid UI development",
  "styled-components": "CSS-in-JS library for component-scoped styles",
  "css modules": "Locally-scoped CSS class names at build time",
  "sass/scss": "CSS preprocessor with variables, nesting, and mixins",
  "chakra ui": "Accessible React component library with a design system",
  "shadcn/ui": "Copy-paste component library built on Radix + Tailwind",

  // State management
  zustand: "Minimal, unopinionated global state for React",
  jotai: "Atomic state management library for React",
  "redux toolkit": "Official opinionated toolset for Redux state management",
  recoil: "Facebook's atomic state management for React",
  "server state": "Server-synchronised state via TanStack Query (React Query)",
  "tanstack query": "Async state management and data fetching for React",

  // Backend / API
  fastify: "High-performance Node.js web framework with schema validation",
  express: "Minimal and flexible Node.js web application framework",
  nestjs: "TypeScript Node.js framework with Angular-inspired architecture",
  hono: "Ultrafast edge-native web framework for Cloudflare Workers",
  axum: "Ergonomic Rust web framework built on Tokio and Tower",
  "actix-web": "High-performance Rust web framework with actor model support",
  tonic: "Rust gRPC framework built on Tokio",

  // Database / ORM
  prisma: "Next-generation TypeScript ORM with a declarative schema",
  drizzle: "Lightweight TypeScript ORM with SQL-like query builder",
  typeorm: "TypeScript ORM supporting Active Record and Data Mapper patterns",
  mongoose: "MongoDB object modelling for Node.js",
  sqlx: "Async Rust SQL toolkit with compile-time query verification",
  diesel: "Safe, extensible Rust ORM and query builder",
  "sea-orm": "Async Rust ORM built on SQLx with ActiveRecord pattern",

  // Testing
  vitest: "Vite-native unit test runner with Jest-compatible API",
  jest: "JavaScript testing framework with zero config and built-in mocking",
  testing: "Automated test suite covering unit and integration scenarios",
  playwright: "End-to-end browser testing with multi-browser support",
  cypress: "End-to-end testing framework with real browser automation",
  storybook: "Component development and visual testing in isolation",

  // GraphQL
  graphql: "Query language and runtime for APIs with a typed schema",
  "apollo client": "State management and data-fetching library for GraphQL",
  "graphql schema-first":
    "Schema-first GraphQL development with `.graphql` files",

  // Infrastructure
  docker: "Containerised deployment with reproducible environments",
  "ci/cd": "Automated build, test, and deployment pipeline",
  kubernetes: "Container orchestration for scalable deployments",

  // Runtime / async
  tokio: "Asynchronous Rust runtime for writing reliable network services",
  "node.js": "JavaScript runtime built on V8 for server-side execution",
  bun: "All-in-one JavaScript runtime, bundler, and package manager",
  deno: "Secure TypeScript runtime with native ES module support",

  // Other
  ssr: "Server-side rendering for improved performance and SEO",
  websockets:
    "Bidirectional real-time communication over a persistent connection",
  trpc: "End-to-end typesafe API layer without code generation",
  zod: "TypeScript-first schema validation with static type inference",
  stripe: "Payment processing and subscription management API",
  wasm: "WebAssembly — near-native performance in browser and server runtimes",
  monorepo: "Multi-package workspace managed with shared tooling",
  "component architecture": "Reusable component-driven UI development pattern",
  "controller pattern":
    "Request-handling layer separating routing from business logic",
  "service layer":
    "Encapsulated business logic separated from transport concerns",
  "data modelling":
    "Structured schema and model definitions for domain entities",
};

/** Look up a skill description, falling back to a generic phrase. */
export function describeSkill(skillName: string): string {
  const key = skillName.toLowerCase();
  return (
    SKILL_DESCRIPTIONS[key] ?? `${skillName} integration and best practices`
  );
}

// ── Role content blocks ───────────────────────────────────────────────────────
// Per-role static text used as template variables.
// These are the sections that DON'T change based on the detected stack.

export interface RoleContent {
  identity: string;
  personality: string;
  memory: string;
  experience: string;
}

const ROLE_CONTENT: Record<string, RoleContent> = {
  "Fullstack Engineer": {
    identity:
      "A generalist engineer who owns the complete request lifecycle — from React component to database query. Comfortable switching context between UI and server without losing momentum.",
    personality:
      "Pragmatic and delivery-focused. Prefers working solutions over architectural purity, but won't cut corners that cause maintenance pain. Communicates clearly about trade-offs.",
    memory:
      "Tracks the boundary between frontend and backend contracts. Remembers which API routes exist, what shape their responses take, and which UI components depend on them.",
    experience:
      "Has shipped multiple full-stack features end-to-end. Knows where full-stack bugs hide: serialisation mismatches, stale caches, and N+1 queries triggered by innocent-looking UI changes.",
  },
  "Frontend Engineer": {
    identity:
      "A specialist in building fast, accessible, and maintainable user interfaces. Deeply familiar with component architecture, design systems, and client-side performance.",
    personality:
      "Detail-oriented and user-empathetic. Cares about pixel precision and interaction quality. Pushes back on designs that are hard to implement accessibly.",
    memory:
      "Maintains a mental map of the component hierarchy, shared design tokens, and which components are pure vs stateful. Remembers performance hot spots and rerender causes.",
    experience:
      "Has optimised render performance, built reusable design system components, and debugged hydration mismatches in SSR apps.",
  },
  "Backend API Engineer": {
    identity:
      "An engineer focused on building reliable, well-validated HTTP APIs. Owns request validation, error handling, service layer design, and database access patterns.",
    personality:
      "Systematic and defensive. Assumes all input is malformed until validated. Values explicit error messages over silent failures. Documents contracts clearly.",
    memory:
      "Tracks the API surface: routes, request/response schemas, auth requirements, and error codes. Remembers which endpoints are performance-sensitive.",
    experience:
      "Has designed REST and RPC APIs, written middleware for auth and rate limiting, and debugged production latency issues traced back to missing database indexes.",
  },
  "QA & Testing Engineer": {
    identity:
      "An engineer dedicated to test strategy, coverage quality, and preventing regressions. Owns the test pyramid and quality gates in CI.",
    personality:
      "Methodical and sceptical. Questions happy-path assumptions. Finds edge cases others miss. Treats flaky tests as bugs, not noise.",
    memory:
      "Remembers which parts of the codebase have weak coverage, which tests are slow or flaky, and what caused previous regressions.",
    experience:
      "Has set up testing infrastructure from scratch, migrated between testing frameworks, and built E2E test suites that catch real bugs without slowing CI below 10 minutes.",
  },
  "Database Engineer": {
    identity:
      "A specialist in schema design, query optimisation, and data integrity. Owns the database layer from migrations to production query plans.",
    personality:
      "Rigorous and long-term focused. Thinks about schema changes in terms of migration safety and rollback paths. Resists shortcuts that create data debt.",
    memory:
      "Tracks the schema history, knows which queries are slow under load, and remembers the reasoning behind non-obvious index choices.",
    experience:
      "Has designed schemas that evolved gracefully over 3+ years, written zero-downtime migrations, and diagnosed production slowdowns using query plan analysis.",
  },
  "DevOps Engineer": {
    identity:
      "An engineer who owns the path from code commit to production. Builds and maintains CI/CD pipelines, container infrastructure, and deployment automation.",
    personality:
      "Reliability-focused and automation-first. Hates manual steps in deployment. Treats infrastructure as code and runbooks as a last resort.",
    memory:
      "Tracks the deployment topology, environment variable requirements, and the history of production incidents and their root causes.",
    experience:
      "Has built CI pipelines that run in under 5 minutes, containerised Node.js and Rust services, and set up staging environments that mirror production faithfully.",
  },
  "GraphQL Engineer": {
    identity:
      "A specialist in GraphQL schema design, resolver implementation, and client-server contract management.",
    personality:
      "Schema-first and contract-driven. Thinks in graphs before thinking in code. Values backwards compatibility and deprecation discipline.",
    memory:
      "Tracks the schema evolution, deprecated fields, and which resolvers have N+1 problems. Remembers the reasoning behind non-obvious schema design decisions.",
    experience:
      "Has designed GraphQL schemas that served mobile and web clients simultaneously, implemented DataLoader for batching, and built schema stitching for federated graphs.",
  },
  "Platform / Monorepo Engineer": {
    identity:
      "An engineer focused on developer experience, build tooling, and package interdependencies in a monorepo workspace.",
    personality:
      "Systems-thinker with strong opinions about developer ergonomics. Measures success in build times and how quickly new engineers become productive.",
    memory:
      "Tracks the dependency graph between packages, which scripts are slow, and what the shared tooling configuration looks like.",
    experience:
      "Has set up pnpm workspaces, configured shared TypeScript and lint configs, and implemented Turborepo pipelines that reduce CI time by caching unchanged packages.",
  },
  "Developer Tooling Engineer": {
    identity:
      "An engineer who builds the tools other engineers use — CLIs, code generators, IDE integrations, and automation scripts.",
    personality:
      "Pragmatic and ergonomics-obsessed. Treats other developers as the end users. Values clear error messages and fast feedback loops above all else.",
    memory:
      "Tracks the tool's distribution model, platform compatibility matrix, and known edge cases in the binary resolution logic.",
    experience:
      "Has built CLIs distributed via npm, written cross-platform shell scripts, and debugged path resolution issues across Linux, macOS, and Windows.",
  },
  "WebAssembly Engineer": {
    identity:
      "An engineer who compiles Rust (or other systems languages) to WebAssembly and integrates it with JavaScript runtimes.",
    personality:
      "Performance-obsessed and boundary-aware. Thinks carefully about the Wasm↔JS boundary overhead and chooses what to cross it wisely.",
    memory:
      "Tracks the WASM module API surface, serialisation costs at the JS boundary, and which operations benefit from native performance.",
    experience:
      "Has compiled Rust crates to WASM, written wasm-bindgen bindings, and measured the performance differential between JS and WASM implementations.",
  },
};

const DEFAULT_ROLE_CONTENT: RoleContent = {
  identity:
    "A skilled engineer contributing to this project with expertise in the detected tech stack.",
  personality:
    "Collaborative, pragmatic, and delivery-focused. Values clear communication and well-tested code.",
  memory:
    "Retains context about architectural decisions, common failure modes, and the reasoning behind key design choices in this codebase.",
  experience:
    "Has worked on projects using the detected tech stack and understands the common patterns and pitfalls involved.",
};

/** Get static role content, falling back to defaults for unknown roles. */
export function getRoleContent(role: string): RoleContent {
  return ROLE_CONTENT[role] ?? DEFAULT_ROLE_CONTENT;
}

// ── Stack summary ─────────────────────────────────────────────────────────────

/** Build a human-readable stack summary line for use in templates. */
export function buildStackSummary(context: RepoContext): string {
  const { stack } = context;
  const parts: string[] = [stack.language];

  if (stack.framework) parts.push(stack.framework);
  if (stack.database) parts.push(stack.database);
  if (stack.testing) parts.push(stack.testing);
  if (stack.runtime && stack.runtime !== stack.language)
    parts.push(stack.runtime);

  return parts.join(" · ");
}

/** Build bullet-point skill lines with descriptions for the template. */
export function buildSkillLines(
  context: RepoContext,
  minConfidence = 0.5,
): string {
  return context.stack.skills
    .filter((s) => s.confidence >= minConfidence && !s.name.startsWith("__"))
    .map((s) => {
      const desc = describeSkill(s.name);
      const badge = s.confidence >= 0.9 ? "" : " *(detected)*";
      return `- **${s.name}**${badge}: ${desc}`;
    })
    .join("\n");
}

/** Build an infra flags line (Docker, CI, Monorepo). */
export function buildInfraLine(context: RepoContext): string {
  const flags: string[] = [];
  if (context.architecture.has_docker) flags.push("Docker");
  if (context.architecture.has_ci) flags.push("CI/CD pipeline");
  if (context.architecture.has_monorepo) flags.push("Monorepo workspace");
  if (context.architecture.has_git) flags.push("Git");
  return flags.length > 0
    ? flags.join(", ")
    : "No infrastructure tooling detected";
}
