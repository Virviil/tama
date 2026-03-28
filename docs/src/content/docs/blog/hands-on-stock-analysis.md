---
title: "Hands-on: 4 AI analysts researching a stock in parallel"
date: 2026-03-28 00:00:00
authors:
  - tama
tags:
  - hands-on
  - agents
  - tutorial
excerpt: >
  We ported CrewAI's stock analysis tutorial to tama. The result: 4 specialist AI analysts running in parallel, synthesized by a portfolio manager — defined entirely in Markdown files.
---

Let's build something real.

Four specialist AI analysts — fundamental, technical, sentiment, risk — run in parallel on any stock ticker. A portfolio manager reads all four reports and produces a final BUY / HOLD / SELL recommendation with price targets.

This is `examples/18-stock-analysis` from the tama repo, ported from [CrewAI's stock analysis tutorial](https://github.com/crewAIInc/crewAI-examples/tree/main/stock_analysis). Same problem, different approach.

## The architecture

```
tamad "Should I invest in NVDA?"
      │
      ▼
  pipeline (FSM)
      │
      ├─► analysts (scatter → 4 parallel workers)
      │       ├─► analyst: FUNDAMENTAL
      │       ├─► analyst: TECHNICAL
      │       ├─► analyst: SENTIMENT
      │       └─► analyst: RISK
      │
      └─► portfolio-manager (synthesize → BUY/HOLD/SELL)
```

Three patterns working together: **FSM** connects the two phases, **scatter** fans out to the four analysts, each analyst runs a **react** loop with web search.

## The files

Five `AGENT.md` files. That's the whole project.

### `agents/pipeline/AGENT.md`

```yaml
---
name: pipeline
description: Stock analysis pipeline — 4 specialist analysts, then a portfolio manager.
version: 1.0.0
pattern: fsm
initial: analysts
states:
  analysts: portfolio-manager
  portfolio-manager: ~
---
```

This is the entry point. It's pure routing — no prompt body needed.

### `agents/analysts/AGENT.md`

```yaml
---
name: analysts
description: Runs 4 specialist analysts in parallel for a given stock ticker.
version: 1.0.0
pattern: scatter
worker: analyst
call:
  model:
    role: thinker
  uses:
    - search-web
    - mem-set
---

You are a research director coordinating 4 specialist analysts.

Call start() to receive the stock ticker and question.

1. Extract the ticker symbol.
2. Store it: tama_mem_set("ticker", "<TICKER>")
3. Call parallel_run with 4 tasks:
   [
     "FUNDAMENTAL: <TICKER> — revenue, margins, P/E, cash flow, moat",
     "TECHNICAL: <TICKER> — trend, RSI, moving averages, support/resistance",
     "SENTIMENT: <TICKER> — news, analyst ratings, insider activity",
     "RISK: <TICKER> — macro, competition, regulation, valuation"
   ]
4. Store the combined reports: tama_mem_set("analyst_reports", "<all 4>")

Call finish(key="done", value="<brief synthesis>").
```

### `agents/analyst/AGENT.md`

```yaml
---
name: analyst
description: One specialist analyst — researches one dimension of a stock.
version: 1.0.0
pattern: react
call:
  model:
    role: thinker
  uses:
    - search-web
    - mem-append
---

You are a specialist financial analyst. Your task arrives prefixed with your type:
"FUNDAMENTAL: NVDA — ..."

Run 2–3 targeted searches for your specialty. Extract concrete data points.
Output in this format:

## <TYPE> Analysis: <TICKER>
**Verdict**: Bullish / Bearish / Neutral
**Key findings**: (3 bullet points with data)
**Summary**: (2–3 sentences)

Store your analysis: tama_mem_append("analyst_reports", "<your analysis>")
Call finish(key="done", value="<your formatted analysis>").
```

### `agents/portfolio-manager/AGENT.md`

```yaml
---
name: portfolio-manager
description: Synthesizes 4 analyst reports into a final investment recommendation.
version: 1.0.0
pattern: react
call:
  model:
    role: thinker
  uses:
    - mem-get
---

You are a senior portfolio manager. Read all 4 analyst reports and produce a
final recommendation.

1. tama_mem_get("ticker") and tama_mem_get("analyst_reports")
2. Weigh fundamental (30%), technical (25%), sentiment (20%), risk (25%)
3. Output:

# Stock Analysis: <TICKER>
## Recommendation: BUY / HOLD / SELL
**Conviction**: High / Medium / Low
## Key Drivers (bulls vs bears)
## Analyst Scorecard (table)
## Price Target & Entry
## Caveats

Call finish(key="done", value="<your complete report>").
```

## Run it

```bash
git clone https://github.com/mlnja/tama
cd tama/examples/18-stock-analysis

export ANTHROPIC_API_KEY=sk-ant-...
export TAMA_MODEL_THINKER=anthropic:claude-sonnet-4-6

cargo run --bin tamad -- "Should I invest in NVDA?"
```

You'll see 4 analysts spinning up in parallel, each running its own web search loop. A few minutes later, the portfolio manager synthesizes everything into a structured report.

## What the output looks like

```
# Stock Analysis: NVDA

## Recommendation: BUY
**Conviction**: High

## Key Drivers

### Supporting (Bulls)
- Dominant AI infrastructure position: 80%+ datacenter GPU market share
- Revenue up 122% YoY in latest quarter, margins expanding
- Technical: clean uptrend, RSI 58, holding above 200-day MA

### Against (Bears)
- Valuation stretched: P/E ~35x forward, premium to peers
- Export controls risk to China market (historically ~20% of revenue)
- Sentiment crowded: high short-term expectations

## Analyst Scorecard
| Dimension    | Signal   | Weight |
|-------------|----------|--------|
| Fundamental | Bullish  | 30%    |
| Technical   | Bullish  | 25%    |
| Sentiment   | Neutral  | 20%    |
| Risk        | Bearish  | 25%    |

## Price Target & Entry
- **Target**: $165 (+18% from current ~$140)
- **Entry**: Scale in on pullbacks toward $125–130
- **Stop-loss**: $118 (below key support)
- **Time horizon**: 12 months
```

## What you'd change

**Swap models.** Use a faster/cheaper model for the 4 analysts and a stronger one for the portfolio manager — all without touching the agent files:

```bash
export TAMA_MODEL_THINKER=anthropic:claude-haiku-4-5
export TAMA_MODEL_THINKER_PM=anthropic:claude-opus-4-6
```

Or set it directly in `tama.toml`:

```toml
[models]
thinker = "anthropic:claude-haiku-4-5-20251001"
```

**Add a 5th analyst.** Create `agents/macro-analyst/AGENT.md` with `pattern: react`, add `"MACRO: <TICKER> — interest rates, dollar strength, sector rotation"` to the `parallel_run` call. Done.

**Change the output format.** Edit the portfolio manager's prompt. The format is plain text in the AGENT.md body — it's a diff in git.

**Use a different ticker.** `tamad "What about MSFT?"`. The agents extract the ticker from the input.

## vs the Python version

The CrewAI version of this example is ~250 lines of Python across multiple files — agent class definitions, task objects, crew configuration, tool wrappers. If you want to change the output format, you edit a Python string embedded in a class method.

In tama: 5 Markdown files. Change the output format by editing the prompt. Add an analyst by adding a file. Diff your agent's behavior in git.

---

The full example is at [`examples/18-stock-analysis`](https://github.com/mlnja/tama/tree/main/examples/18-stock-analysis). There are 22 more examples in the repo covering every pattern.

- [Quickstart](/getting-started/quickstart) — your first agent in 5 minutes
- [Patterns overview](/patterns/overview) — all 13 patterns
- [GitHub](https://github.com/mlnja/tama) — source, issues, discussions
