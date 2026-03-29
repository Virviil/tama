---
title: tama_http
description: Built-in HTTP tools — tama_http_get and tama_http_post.
---

HTTP tools let agents fetch URLs and post data to APIs. They must be unlocked via a skill.

## tama_http_get

Fetches the content of a URL via HTTP GET.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `url` | string | The URL to fetch |

**Returns** the full response body as a string.

**Example skill**

```markdown
---
name: fetch-url
description: Fetch the content of any URL.
tools: [tama_http_get]
---

Call tama_http_get with the full URL including query parameters.
The response body is returned as a string. For HTML pages, extract
relevant text by looking for content inside `<p>`, `<article>`, or
`<main>` tags.
```

## tama_http_post

Sends an HTTP POST request with a JSON body.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `url` | string | The URL to post to |
| `body` | string | JSON body as a string |

**Returns** the full response body as a string.

**Example skill**

```markdown
---
name: call-api
description: Call a JSON API endpoint with a POST request.
tools: [tama_http_post]
---

Use tama_http_post(url, body) where body is a JSON string.
Example: tama_http_post("https://api.example.com/data", '{"key": "value"}')
```

## Common pattern: web search

The canonical `search-web` skill combines both HTTP tools:

```markdown
---
name: search-web
description: Search the web using DuckDuckGo. No API key required.
tools: [tama_http_get, tama_files_write]
---

Call tama_http_get with this URL pattern:
  https://html.duckduckgo.com/html/?q=<url-encoded-query>

Encode spaces as `+`. Extract results from `<a class="result__a">` and
`<span class="result__snippet">`. Follow interesting URLs with another
tama_http_get call to read the full page.
```
