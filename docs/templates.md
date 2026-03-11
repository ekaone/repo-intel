# Fallback Templates

When running with `--no-ai`, repo-intel generates documentation using static Handlebars templates. This mode is fast, deterministic, and requires no API keys.

## Default Template Variables

| Variable | Description |
|----------|-------------|
| `{{title}}` | Agent role title |
| `{{repoName}}` | Repository name |
| `{{description}}` | Role description |
| `{{responsibilities}}` | Array of responsibility strings |
| `{{patterns}}` | Array of stack-specific pattern strings |
| `{{tasks}}` | Array of common task strings |

## Skill Descriptions

The static generator maps detected skill names to concise pattern descriptions. These are defined in [`packages/repo-intel/src/fallback/static-gen.ts`](../packages/repo-intel/src/fallback/static-gen.ts).

To add descriptions for additional skills, extend the `SKILL_DESCRIPTIONS` record:

```typescript
// static-gen.ts
export const SKILL_DESCRIPTIONS: Record<string, string> = {
  "my-framework": "Use my-framework patterns: ...",
  // ...
};
```

## Customising Templates

The base template is exported from `templates.ts`. To override, you can copy and modify the Handlebars template string and re-compile with `Handlebars.compile(yourTemplate)`.

Future versions will support loading templates from a `.repo-intel/templates/` directory.
