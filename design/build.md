# Build system

## Rust as the compiler and runtime language

**Decision:** `tama` CLI and the runtime inside Docker are one Rust binary.

**Why:**
- `clap` — best CLI parser
- `tokio` — async for parallel workers
- `serde` + `gray_matter` — YAML frontmatter parsing
- Single binary ~5MB with no dependencies
- No `pip install` in the compiler runtime — fast and reproducible

**❌ Anti-pattern:** Python as the compiler language.
That requires installing Python and its dependencies just to build an image
that itself uses Python. A recursive problem.

### Rust crates

```toml
[dependencies]
clap       = { version = "4", features = ["derive"] }   # CLI
tokio      = { version = "1", features = ["full"] }     # async runtime
serde      = { version = "1", features = ["derive"] }   # serialization
serde_yaml = "0.9"                                      # YAML parsing
gray_matter = "0.2"                                     # YAML frontmatter from .md
reqwest    = { version = "0.12", features = ["json"] }  # HTTP for LLM API
anyhow     = "1"                                        # error handling
walkdir    = "2"                                        # walking skills/
```

---

## `tama` and `tamad` — two binaries, two names

Analogy: `javac` (compiler) and `jvm` (runtime) — different things, both part of Java.

| | `tama` | `tamad` |
|---|---|---|
| Role | CLI / compiler | runtime |
| Lives on | developer machine | inside Docker image |
| Platform | `darwin/arm64`, `linux/amd64`, … | always `linux/amd64` |
| Commands | `init`, `add`, `brew`, `lint` | takes task as positional arg |
| LLM | no | yes |

One codebase, two build artifacts. GitHub Releases publishes both:
```
tama-darwin-arm64     ← developer installs on Mac
tama-darwin-x86_64
tama-linux-amd64      ← CLI on Linux
tamad-linux-amd64     ← goes into Docker image
```

**How `tamad` gets into the image:**

`tama brew` downloads `tamad-linux-amd64` for the matching version from GitHub Releases,
caches it in `~/.tama/cache/tamad-linux-amd64-{version}`, places it in a tmpdir build context:

```
GitHub Releases: tamad-linux-amd64-v1.2.3
       ↓ (once, then cached)
~/.tama/cache/tamad-linux-amd64-v1.2.3
       ↓
tama brew → tmpdir/tamad
       ↓
docker build → COPY tamad /tamad
       ↓
ENTRYPOINT ["/tamad"]
```

**Versioning:** `tama` always puts the same version of `tamad` into the image.
No drift between compiler and runtime.

**❌ Anti-pattern:** `current_exe()` — copy itself into the image.
`tama` is built for the developer's platform; the image needs `linux/amd64`.

**❌ Anti-pattern:** requiring `cargo build` from the user.
The user installs a ready-made tool, not a source tree to compile.

---

## Distroless final image

**Decision:** `gcr.io/distroless/cc-debian12` as the final image.
Multi-stage build: `debian:bookworm-slim` as builder → copy only the required binaries.

```
debian:bookworm-slim  →  (COPY --from=builder)  →  gcr.io/distroless/cc-debian12
      builder                                              final ~8MB
```

**Why:** no shell, no package managers, no root, minimal attack surface.
Size ~8MB + deps vs ~80MB for ubuntu.

**Critical consequence:** distroless = no bash. A `router.sh` is impossible.
The Rust binary itself becomes the orchestrator — it invokes binaries via `std::process::Command`.
This is a feature, not a bug.

**For development:** `gcr.io/distroless/cc-debian12:debug` — has a busybox shell,
you can exec in and poke around.

**❌ Anti-pattern:** `router.sh` as entrypoint.
A bash script inside distroless won't run. All orchestration belongs in the Rust binary.

**❌ Anti-pattern:** ubuntu or debian-slim as the final image.
They have a shell, apt, curl — huge attack surface and 70MB of waste.

---

## `tama brew` = `docker build` directly, no file

**Decision:** `tama brew` does not write a `Dockerfile` into the project. Instead:
1. Rust generates the Dockerfile content in memory
2. Pipes it to `docker build` via stdin (`docker build -f - .`)
3. Builds the image directly

```rust
// Pseudocode
let dockerfile = generate_dockerfile(...);
Command::new("docker")
    .args(["build", "-t", &image_tag, "-f", "-", "."])
    .stdin(Stdio::piped())
    .spawn()
    // → write dockerfile to stdin
```

**Why:** the Dockerfile is an implementation detail of the compiler, not a project artifact.
The user thinks in terms of "build me an image", not "generate me a Dockerfile".
No risk of committing a stale Dockerfile.

**❌ Anti-pattern:** Dockerfile as a file in the project.
It goes stale when skills change. The user edits it manually and breaks invariants.

---

## Dependencies — declared in the skill, installed at compile time

**Decision:** each skill declares `depends` in the `tama:` frontmatter block.
`tama brew` collects the union of all deps, deduplicates, builds the image.

```yaml
tama:
  depends:
    apt:
      - poppler-utils
    uv:
      - pypdf2>=3.0.1
    bins:
      - pdftotext    # compiler checks the binary exists after install
```

**Why:** dependencies are resolved once at build time, not at runtime.
Version conflicts surface immediately, not when the agent is already running.

**❌ Anti-pattern:** `pip install` in skill runtime.
Slow, unreliable, doesn't work in distroless, violates
"dependencies are known before execution".

### apt + distroless: two-phase discovery → single layer

Problem: distroless has no apt. You can't install a package into the final image.
Problem 2: N separate `COPY --from=builder` instructions = N layers in the final image.

**Solution: two-phase process with a single COPY layer.**

**Phase 1 — discovery (Rust runs a temporary container):**
- `dpkg -L poppler-utils` → list of all files in the package
- `ldd /usr/bin/pdftotext` → recursive shared library dependencies
- Merge + deduplicate → complete list of paths

**Phase 2 — Dockerfile with a single layer via `/bundle`:**

```dockerfile
FROM debian:bookworm-slim AS deps
RUN apt-get update && apt-get install -y poppler-utils

FROM debian:bookworm-slim AS collector
COPY --from=deps / /
# Collect ALL required files into /bundle, preserving path structure
RUN mkdir -p /bundle && \
    cp --parents /usr/bin/pdftotext /bundle && \
    cp --parents /usr/lib/x86_64-linux-gnu/libpoppler.so.126 /bundle && \
    # ... all files from dpkg+ldd in a single RUN
    true

FROM gcr.io/distroless/cc-debian12
# ONE COPY = ONE LAYER for all apt deps
COPY --from=collector /bundle/ /
COPY tamad /tamad
ENV TAMA_ENTRYPOINT_AGENT=essay-critic
ENTRYPOINT ["/tamad"]
```

**Key trick:** `cp --parents` in a single `RUN` collects everything into `/bundle` preserving
path structure. Then `COPY --from=collector /bundle/ /` — one instruction, one layer.

**Why a single layer matters:** every `COPY` creates a layer. 200 shared libs = 200 layers
without this trick. One layer = less metadata, faster pull, cleaner image history.

**❌ Anti-pattern:** N separate `COPY --from=builder /usr/lib/libfoo.so /usr/lib/libfoo.so`.
Works, but O(N) layers where N = number of files.

**❌ Anti-pattern:** falling back to `debian:bookworm-slim` when apt deps are present.
That defeats the entire purpose of distroless — attack surface, image size.

### uv + distroless: `--target`

```dockerfile
FROM ghcr.io/astral-sh/uv:debian AS builder
RUN uv pip install --system --target=/deps anthropic>=0.40.0

FROM gcr.io/distroless/python3-debian12
COPY --from=builder /deps /deps
ENV PYTHONPATH=/deps
```

`--target` dumps all packages into a single directory without binding to a Python version.
`PYTHONPATH=/deps` — Python finds packages without a venv.

**❌ Anti-pattern:** copying the entire `.venv`.
It contains symlinks, activation scripts, a python binary — all garbage in distroless.
The path to `site-packages` hardcodes the Python version (`lib/python3.11/`) — fragile.
