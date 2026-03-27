# tama — Project Guidelines

## Color Schema

The canonical color palette used across all tama UIs (web, docs, etc).

### Base

| Token | Hex | Usage |
|-------|-----|-------|
| Background | `#ffffff` | Main page background |
| Surface | `#f9fafb` | Panel headers, cards |
| Surface alt | `#f3f4f6` | Code bg, subtle fills |
| Border | `#e5e7eb` | Panel borders, dividers |
| Border subtle | `#f3f4f6` | Row separators |
| Text primary | `#111827` | Main text |
| Text secondary | `#374151` | Body text, results |
| Text muted | `#9ca3af` | Meta, labels, disabled |

### Accent — Sky (primary)

| Token | Hex | Usage |
|-------|-----|-------|
| Sky 50 | `#f0f9ff` | Active row background |
| Sky 100 | `#e0f2fe` | Badge background (input tokens) |
| Sky 500 | `#0ea5e9` | Primary accent, links, active border |

### Accent — Fuchsia (secondary)

| Token | Hex | Usage |
|-------|-----|-------|
| Fuchsia 50 | `#fdf4ff` | Selected row background |
| Fuchsia 500 | `#d946ef` | Secondary accent, selected border, output tokens badge |

### Semantic

| Token | Hex | Usage |
|-------|-----|-------|
| Green 600 | `#16a34a` | Success, ok status, model badge |
| Red 600 | `#dc2626` | Error status |
| Orange 500 | `#f97316` | Tool calls, duration badge, parallel marker |

### Badges

| Class | Background | Text | Meaning |
|-------|-----------|------|---------|
| `.b-model` | `#dcfce7` | `#16a34a` | Model name |
| `.b-temp` | `#f3f4f6` | `#374151` | Temperature |
| `.b-in` | `#e0f2fe` | `#0ea5e9` | Input tokens |
| `.b-out` | `#fdf4ff` | `#d946ef` | Output tokens |
| `.b-dur` | `#fff7ed` | `#f97316` | Duration |

### Typography

UI Font: `'Rethink Sans', sans-serif`
Mono Font: `'DM Mono', monospace` (code blocks, inline code)
Base size: `12px`

## Naming Conventions

- **kebab-case everywhere**: agent names, skill names, project names, pattern names
- Only lowercase letters, digits, and hyphens in identifiers

## Key Files

- `web/src/style.css` — canonical color definitions (source of truth)
- `docs/src/styles/custom.css` — Starlight overrides to match web palette
