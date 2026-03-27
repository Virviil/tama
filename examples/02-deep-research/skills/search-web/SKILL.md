---
name: search-web
description: Search the web using DuckDuckGo. Use when you need to find current information, facts, news, or any topic online. No API key required.
tools: [http_get, write_file]
---

Search the web using DuckDuckGo HTML search (no API key needed).

## How to search

Call http_get with this URL pattern:

  https://html.duckduckgo.com/html/?q=<url-encoded-query>

Encode spaces as `+`, special characters with `%XX`.

Examples:
- http_get("https://html.duckduckgo.com/html/?q=rust+programming+language+2024")
- http_get("https://html.duckduckgo.com/html/?q=climate+change+latest+research")

## Parsing results

The response is HTML. Extract useful content from:
- `<a class="result__a">` — result title and link
- `<span class="result__snippet">` — description snippet

## Going deeper

After finding a relevant URL, call http_get on that URL to read the full page.
Prefer well-known sources: Wikipedia, official docs, reputable news sites.
