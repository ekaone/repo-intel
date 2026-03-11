# AI Providers

repo-intel supports three AI providers. Configure via `.repo-intel.toml` or environment variables.

## Anthropic (default)

**Model:** `claude-sonnet-4-20250514`

```toml
[ai]
provider = "anthropic"
model = "claude-sonnet-4-20250514"  # optional override
```

**Environment variable:**

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
```

Get a key at [console.anthropic.com](https://console.anthropic.com).

---

## OpenAI

**Model:** `gpt-4o`

```toml
[ai]
provider = "openai"
model = "gpt-4o"  # optional override
```

**Environment variable:**

```bash
export OPENAI_API_KEY="sk-..."
```

Get a key at [platform.openai.com](https://platform.openai.com).

---

## Ollama (local/offline)

**Model:** `llama3` (or any model you have pulled)

```toml
[ai]
provider = "ollama"
model = "llama3"
```

**Environment variables (optional):**

```bash
export OLLAMA_BASE_URL="http://localhost:11434"  # default
export OLLAMA_MODEL="llama3"                      # default
```

Pull a model first:

```bash
ollama pull llama3
```

See [ollama.ai](https://ollama.ai) for installation and available models.

---

## No AI (offline static mode)

Use `--no-ai` to skip LLM calls entirely and generate minimal but accurate docs from templates:

```bash
repo-intel generate --no-ai
```

See [templates.md](templates.md) for customising the fallback output.
