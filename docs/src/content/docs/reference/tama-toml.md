---
title: tama.toml
description: Project configuration file reference.
---

`tama.toml` is the project configuration file, created by `tama init` in the project root.

## Full example

```toml
[project]
name = "my-project"
entrypoint = "researcher"

[models]
thinker = { name = "anthropic:claude-opus-4-6",          temperature = 1.0 }
worker  = { name = "anthropic:claude-sonnet-4-6",         temperature = 0.0 }
default = { name = "anthropic:claude-haiku-4-5-20251001" }

# Optional: infrastructure overrides (non-secret, safe to commit)
[providers.anthropic]
base_url = "https://my-proxy.internal/anthropic"

[providers.ollama]
base_url = "http://192.168.1.10:11434"
```

---

## `[project]`

### `name` (required)

```toml
[project]
name = "my-project"
```

Project name. Lowercase letters, digits, hyphens only.

### `entrypoint` (required)

```toml
[project]
entrypoint = "researcher"
```

The agent that runs when you execute:

```bash
tama run "task input"
```

Override at runtime:

```bash
TAMA_ENTRYPOINT_AGENT=summarizer tama run "task input"
```

---

## `[models]`

Maps role names to model configurations. Supports shorthand (name only) and extended forms:

### Shorthand

```toml
[models]
thinker = "anthropic:claude-opus-4-6"
worker  = "anthropic:claude-sonnet-4-6"
default = "anthropic:claude-haiku-4-5-20251001"
```

### Extended

```toml
[models]
thinker = { name = "anthropic:claude-opus-4-6",          temperature = 1.0 }
worker  = { name = "anthropic:claude-sonnet-4-6",         temperature = 0.0, max_tokens = 4096 }
default = { name = "anthropic:claude-haiku-4-5-20251001" }
```

Both forms are valid and can be mixed. Extended fields:

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | `provider:model-id` (required) |
| `temperature` | float | Generation temperature. Overrides role built-in default. |
| `max_tokens` | integer | Max output tokens. Defaults to not set (∞). |
| `base_url` | string | Custom endpoint for self-hosted/proxy. Non-secret; safe to commit. |

### Built-in role defaults

| Role | Temperature | max\_tokens |
|------|------------|------------|
| `thinker` | `1.0` | not set |
| `worker` | `0.0` | not set |
| `default` | `0.8` | not set |

If temperature is not specified in `tama.toml` or env, the built-in default is used. `max_tokens` is never set by default — provider limit applies.

### Pattern defaults

When an agent or step has no `model:` config, tama picks a role by pattern:

| Pattern | Default role |
|---------|-------------|
| `react` | `thinker` |
| All others | `worker` |
| Fallback | `default` |

### Environment variable overrides

Every field can be overridden at runtime. See [Model Configuration](/reference/models) for the full override chains.

Quick reference for role overrides (replace `{ROLE}` with uppercase role name, hyphens → underscores):

```bash
TAMA_MODEL_THINKER_NAME=anthropic:claude-sonnet-4-6
TAMA_MODEL_THINKER_TEMPERATURE=0.5
TAMA_MODEL_THINKER_MAX_TOKENS=8192
TAMA_MODEL_THINKER_API_KEY=sk-ant-...
TAMA_MODEL_THINKER_BASE_URL=https://my-proxy.internal
```

---

## `[providers.*]`

Optional section for provider-level infrastructure config. Non-secret fields only — API keys go in environment variables.

```toml
[providers.anthropic]
base_url = "https://my-proxy.internal/anthropic"

[providers.openai]
base_url = "https://my-openai.openai.azure.com/openai"

[providers.google]
base_url = "https://generativelanguage.googleapis.com"

[providers.ollama]
base_url = "http://192.168.1.10:11434"
```

| Field | Description |
|-------|-------------|
| `base_url` | Custom endpoint. Overridden by `TAMA_PROVIDER_{PROVIDER}_BASE_URL` or `TAMA_MODEL_{ROLE}_BASE_URL`. |

---

## Model format

All model references use `provider:model-id`:

| Provider | Format | Example |
|----------|--------|---------|
| Anthropic | `anthropic:model-id` | `anthropic:claude-opus-4-6` |
| OpenAI | `openai:model-id` | `openai:gpt-4o` |
| Google | `google:model-id` | `google:gemini-2.0-flash` |
| Ollama | `ollama:model-id` | `ollama:qwen2.5:14b` |

---

## Environment variables summary

| Variable | Description |
|----------|-------------|
| `TAMA_ENTRYPOINT_AGENT` | Override `[project].entrypoint` |
| `TAMA_MODEL_{ROLE}_NAME` | Override model name for a role |
| `TAMA_MODEL_{ROLE}_TEMPERATURE` | Override temperature for a role |
| `TAMA_MODEL_{ROLE}_MAX_TOKENS` | Override max_tokens for a role |
| `TAMA_MODEL_{ROLE}_API_KEY` | Override API key for a role |
| `TAMA_MODEL_{ROLE}_BASE_URL` | Override base URL for a role |
| `ANTHROPIC_API_KEY` | Anthropic API key (all Anthropic roles) |
| `OPENAI_API_KEY` | OpenAI API key (all OpenAI roles) |
| `GEMINI_API_KEY` | Google API key (all Google roles) |
| `TAMA_PROVIDER_ANTHROPIC_BASE_URL` | Base URL for all Anthropic roles |
| `TAMA_PROVIDER_OPENAI_BASE_URL` | Base URL for all OpenAI roles |
| `TAMA_PROVIDER_GOOGLE_BASE_URL` | Base URL for all Google roles |
| `TAMA_PROVIDER_OLLAMA_BASE_URL` | Base URL for Ollama |
