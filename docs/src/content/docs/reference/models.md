---
title: Model Configuration
description: Complete reference for model roles, providers, and the full override chain for every parameter.
---

tama uses a **role-based model system**. Roles define named model configurations (model name, temperature, max_tokens) that agents and steps reference by name. You configure roles in `tama.toml` and override any field at runtime via environment variables.

## Built-in roles

Three roles have opinionated defaults:

| Role | Intent | Default temperature | Default max\_tokens |
|------|--------|--------------------|--------------------|
| `thinker` | Deep reasoning, multi-step planning, creative tasks | `1.0` | not set (∞) |
| `worker` | Deterministic output, tool use, structured responses | `0.0` | not set (∞) |
| `default` | Catch-all fallback when no role is specified | `0.8` | not set (∞) |

You only need to define the roles you use. A project with a single role `default` is perfectly valid.

## Pattern defaults

When an agent or step has no `model:` config, tama picks a role based on the execution pattern:

| Pattern | Default role |
|---------|-------------|
| `react` | `thinker` |
| Everything else | `worker` |
| Agent (no pattern match) | `default` |

If the pattern-default role is not defined in your `tama.toml`, tama falls back to `default`. If `default` is also missing, startup fails with a clear error.

---

## tama.toml configuration

### Shorthand (name only)

```toml
[models]
thinker = "anthropic:claude-opus-4-6"
worker  = "anthropic:claude-sonnet-4-6"
default = "anthropic:claude-haiku-4-5-20251001"
```

### Extended (with overrides)

```toml
[models]
thinker = { name = "anthropic:claude-opus-4-6", temperature = 1.0 }
worker  = { name = "anthropic:claude-sonnet-4-6", temperature = 0.0, max_tokens = 4096 }
default = { name = "anthropic:claude-haiku-4-5-20251001" }
```

Both forms can be mixed in the same file. `base_url` is also supported for self-hosted models:

```toml
[models]
default = { name = "ollama:qwen2.5:14b", base_url = "http://192.168.1.10:11434" }
```

### Provider infrastructure

Non-secret infrastructure config (base URLs for proxies, Azure, LiteLLM, custom Ollama endpoints) goes in `[providers.*]`:

```toml
[providers.anthropic]
base_url = "https://my-proxy.internal/anthropic"

[providers.openai]
base_url = "https://my-openai.openai.azure.com/openai"

[providers.ollama]
base_url = "http://192.168.1.10:11434"
```

**No API keys in `tama.toml`** — use environment variables only.

---

## Agent / step frontmatter

```yaml
# Reference a named role
call:
  model:
    role: thinker

# Named role with local overrides
call:
  model:
    role: worker
    temperature: 0.3
    max_tokens: 512

# Direct model spec (no role lookup)
call:
  model:
    name: anthropic:claude-opus-4-6
    temperature: 0.5
    max_tokens: 4096
```

When `name` and `role` are both present, `name` takes precedence.

---

## Parameter reference

Each parameter shows its full resolution chain — highest priority first.

### Model name

```
agent/step frontmatter  model: name:
  → TAMA_MODEL_{ROLE}_NAME
    → [models.{role}].name in tama.toml
      → error: model name is required
```

### Temperature

```
agent/step frontmatter  model: temperature:
  → TAMA_MODEL_{ROLE}_TEMPERATURE
    → [models.{role}].temperature in tama.toml
      → role built-in default (thinker: 1.0 | worker: 0.0 | default: 0.8)
```

### max\_tokens

```
agent/step frontmatter  model: max_tokens:
  → TAMA_MODEL_{ROLE}_MAX_TOKENS
    → [models.{role}].max_tokens in tama.toml
      → not set — provider/model limit applies (effectively ∞)
```

### API key

```
TAMA_MODEL_{ROLE}_API_KEY
  → ANTHROPIC_API_KEY  (or OPENAI_API_KEY / GEMINI_API_KEY for other providers)
    → error: API key required for provider '{provider}'
```

Ollama: no API key required.

### Base URL

```
TAMA_MODEL_{ROLE}_BASE_URL
  → TAMA_PROVIDER_{PROVIDER}_BASE_URL
    → [providers.{provider}].base_url in tama.toml
      → built-in provider default
```

---

## Environment variable reference

### Role-level overrides

Replace `{ROLE}` with the uppercased role name (hyphens become underscores).

| Variable | Description |
|----------|-------------|
| `TAMA_MODEL_{ROLE}_NAME` | Model name for this role (e.g. `anthropic:claude-sonnet-4-6`) |
| `TAMA_MODEL_{ROLE}_TEMPERATURE` | Temperature override (e.g. `0.5`) |
| `TAMA_MODEL_{ROLE}_MAX_TOKENS` | Max tokens override (e.g. `8192`) |
| `TAMA_MODEL_{ROLE}_API_KEY` | API key for this role (any provider) |
| `TAMA_MODEL_{ROLE}_BASE_URL` | Base URL override for this role |

**Examples:**

```bash
TAMA_MODEL_THINKER_NAME=anthropic:claude-sonnet-4-6
TAMA_MODEL_THINKER_TEMPERATURE=0.5
TAMA_MODEL_THINKER_MAX_TOKENS=8192
TAMA_MODEL_THINKER_API_KEY=sk-ant-...
TAMA_MODEL_THINKER_BASE_URL=https://my-proxy.internal/anthropic

# Hyphen → underscore:  role "my-writer" → TAMA_MODEL_MY_WRITER_NAME
```

### Provider-level overrides

| Variable | Description |
|----------|-------------|
| `ANTHROPIC_API_KEY` | API key for all Anthropic roles (unless overridden per-role) |
| `OPENAI_API_KEY` | API key for all OpenAI roles |
| `GEMINI_API_KEY` | API key for all Google roles |
| `TAMA_PROVIDER_ANTHROPIC_BASE_URL` | Base URL for all Anthropic roles |
| `TAMA_PROVIDER_OPENAI_BASE_URL` | Base URL for all OpenAI roles |
| `TAMA_PROVIDER_GOOGLE_BASE_URL` | Base URL for all Google roles |
| `TAMA_PROVIDER_OLLAMA_BASE_URL` | Base URL for Ollama (default: `http://localhost:11434`) |

---

## Supported providers

### Anthropic

```bash
export ANTHROPIC_API_KEY=sk-ant-...
```

| Model | ID |
|-------|----|
| Claude Opus 4.6 | `anthropic:claude-opus-4-6` |
| Claude Sonnet 4.6 | `anthropic:claude-sonnet-4-6` |
| Claude Haiku 4.5 | `anthropic:claude-haiku-4-5-20251001` |

### OpenAI

```bash
export OPENAI_API_KEY=sk-...
```

| Model | ID |
|-------|----|
| GPT-4o | `openai:gpt-4o` |
| GPT-4o mini | `openai:gpt-4o-mini` |

### Google

```bash
export GEMINI_API_KEY=...
```

| Model | ID |
|-------|----|
| Gemini 2.0 Flash | `google:gemini-2.0-flash` |
| Gemini 2.5 Pro | `google:gemini-2.5-pro` |

### Ollama (local)

No API key required. Run models locally:

```bash
ollama pull qwen2.5:14b
```

```toml
[models]
default = "ollama:qwen2.5:14b"
```

To use a non-standard Ollama endpoint:

```bash
export TAMA_PROVIDER_OLLAMA_BASE_URL=http://192.168.1.10:11434
```

or in `tama.toml`:

```toml
[providers.ollama]
base_url = "http://192.168.1.10:11434"
```

---

## Examples

### Minimal single-model project

```toml
[models]
default = "anthropic:claude-sonnet-4-6"
```

All agents and steps use `claude-sonnet-4-6`. No API key in the file — set `ANTHROPIC_API_KEY`.

### Differentiated roles

```toml
[models]
thinker = "anthropic:claude-opus-4-6"
worker  = "anthropic:claude-haiku-4-5-20251001"
default = "anthropic:claude-sonnet-4-6"
```

React agents automatically use `thinker`; oneshot steps use `worker`; anything unspecified falls back to `default`.

### Routing thinker to a proxy

```toml
[models]
thinker = { name = "anthropic:claude-opus-4-6", temperature = 1.0 }
worker  = { name = "anthropic:claude-sonnet-4-6" }
default = { name = "anthropic:claude-haiku-4-5-20251001" }
```

```bash
# Only thinker goes through the proxy; other roles use standard endpoint
export TAMA_MODEL_THINKER_BASE_URL=https://litellm.internal/anthropic
export TAMA_MODEL_THINKER_API_KEY=sk-proxy-...
```

### Ollama for local development

```toml
[models]
default = "ollama:qwen2.5:14b"
```

No API key needed. Works offline.
