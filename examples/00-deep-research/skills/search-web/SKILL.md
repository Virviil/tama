---
name: search-web
description: Search the web via Jina AI — clean markdown results. Requires JINA_API_KEY env var.
tools: [tama_http_get]
---

EVERY call to tama_http_get in this skill MUST include the authorization header. No exceptions:

  headers=[{"Authorization": "Bearer ${JINA_API_KEY}"}]

Calls without this header will return 401 and fail.

## Step 1 — search

  https://s.jina.ai/?q=<url-encoded-query>

Returns a clean list of results: titles, URLs, and snippets. Encode spaces as `+`.

  tama_http_get(url="https://s.jina.ai/?q=rust+programming+language+2024", headers=[{"Authorization": "Bearer ${JINA_API_KEY}"}])

Pick the 1-2 most relevant URLs from the results.

## Step 2 — read

  https://r.jina.ai/<full-url>

Fetches a page as clean markdown — no ads, no nav, no boilerplate.

  tama_http_get(url="https://r.jina.ai/https://en.wikipedia.org/wiki/Rust_(programming_language)", headers=[{"Authorization": "Bearer ${JINA_API_KEY}"}])

## Rules

- At most 1 search call per query.
- At most 2 reader calls per search. Stop when you have enough.
- Extract only what is relevant to your research question.
