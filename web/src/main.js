import Alpine from 'alpinejs'

window.Alpine = Alpine

Alpine.data('app', () => ({
  runs: [],
  selectedRunId: null,
  tree: [],         // AgentGroup[]
  expanded: {},     // groupKey → bool
  activeAttempts: {},// groupKey → attemptIdx
  timeline: null,   // { groupKey, agentName, pattern, spanId, attemptIdx, totalAttempts, events[] }

  // ── Computed: flatten tree into visible rows ──────────────────────────────
  get flatRows() {
    const rows = []
    const walk = (groups, depth) => {
      for (const group of groups) {
        const key = groupKey(group, depth)
        const attemptIdx = this.activeAttempts[key] ?? 0
        const attempt = group.attempts[attemptIdx] ?? group.attempts[0]
        const hasChildren = attempt.children.length > 0
        rows.push({ key, group, depth, attemptIdx, hasChildren, dur: attempt.duration_ms })
        if (hasChildren && this.expanded[key]) {
          walk(attempt.children, depth + 1)
        }
      }
    }
    walk(this.tree, 0)
    return rows
  },

  // ── Init ──────────────────────────────────────────────────────────────────
  async init() {
    await this.loadRuns()
    setInterval(() => this.loadRuns(), 3000)
  },

  async loadRuns() {
    try {
      const res = await fetch('/api/runs')
      this.runs = await res.json()
    } catch (e) {
      console.error('failed to load runs', e)
    }
  },

  // ── Select a run: load tree ───────────────────────────────────────────────
  async selectRun(run) {
    if (this.selectedRunId === run.trace_id) return
    this.selectedRunId = run.trace_id
    this.tree = []
    this.expanded = {}
    this.activeAttempts = {}
    this.timeline = null

    try {
      const res = await fetch(`/api/runs/${run.trace_id}/tree`)
      const tree = await res.json()
      this.tree = tree
      // auto-expand root
      if (tree.length > 0) {
        const key = groupKey(tree[0], 0)
        this.expanded[key] = true
      }
    } catch (e) {
      console.error('failed to load tree', e)
    }
  },

  // ── Toggle expand ─────────────────────────────────────────────────────────
  // (click on row body toggles expand, separate from timeline open)
  toggleExpand(key) {
    this.expanded[key] = !this.expanded[key]
  },

  // ── Select agent: open timeline ───────────────────────────────────────────
  async selectAgent(row) {
    // Toggle expand for agents with children
    if (row.hasChildren) {
      this.expanded[row.key] = !this.expanded[row.key]
    }
    await this.loadTimeline(row.key, row.group, row.attemptIdx)
  },

  async loadTimeline(key, group, attemptIdx) {
    const attempt = group.attempts[attemptIdx] ?? group.attempts[0]
    try {
      const res = await fetch(`/api/agents/${attempt.span_id}/timeline`)
      const events = await res.json()
      const llmEvents = events.filter(e => e.kind === 'llm')
      const firstLlm = llmEvents[0]
      this.timeline = {
        groupKey: key,
        agentName: group.name,
        pattern: group.pattern,
        spanId: attempt.span_id,
        attemptIdx,
        totalAttempts: group.attempts.length,
        events,
        model: firstLlm?.model || '',
        role: firstLlm?.role ?? null,
        temperature: firstLlm?.temperature ?? null,
        systemPrompt: firstLlm?.system_prompt || '',
        totalIn:  llmEvents.reduce((s, e) => s + e.input_tokens,  0),
        totalOut: llmEvents.reduce((s, e) => s + e.output_tokens, 0),
        totalMs:  llmEvents.reduce((s, e) => s + e.duration_ms,   0),
      }
    } catch (e) {
      console.error('failed to load timeline', e)
    }
  },

  // ── Switch retry tab in tree ──────────────────────────────────────────────
  async switchAttempt(key, idx) {
    this.activeAttempts[key] = idx
    // If this agent's timeline is open, reload it
    if (this.timeline && this.timeline.groupKey === key) {
      const row = this.flatRows.find(r => r.key === key)
      if (row) await this.loadTimeline(key, row.group, idx)
    }
  },

  // ── Switch retry tab in timeline header ───────────────────────────────────
  async switchAttemptTimeline(idx) {
    if (!this.timeline) return
    const row = this.flatRows.find(r => r.key === this.timeline.groupKey)
    if (!row) return
    this.activeAttempts[this.timeline.groupKey] = idx
    await this.loadTimeline(this.timeline.groupKey, row.group, idx)
  },
}))

// ── exposed helper for template ───────────────────────────────────────────────
window.prettyJson = function(s) {
  try { return JSON.stringify(JSON.parse(s), null, 2) } catch { return s || '' }
}

window.shortArgs = function(s) {
  try {
    const o = JSON.parse(s)
    const keys = Object.keys(o)
    if (keys.length === 0) return ''
    return keys.map(k => {
      const v = String(o[k])
      return `${k}=${v.length > 30 ? v.slice(0, 30) + '…' : v}`
    }).join('  ')
  } catch { return s || '' }
}

window.truncate = function(s, n) {
  if (!s) return ''
  return s.length > n ? s.slice(0, n) + '…' : s
}

// pattern → color from the original SVG palette + sky/fuchsia accents
const PATTERN_COLORS = {
  react:                  '#3fa9f5',
  scatter:                '#9263ab',
  parallel:               '#3737aa',
  orchestrator:           '#19838e',
  fsm:                    '#8b9b29',
  critic:                 '#f15a24',
  reflexion:              '#aa2e2e',
  constitutional:         '#6b7280',
  debate:                 '#d946ef',
  best_of_n:              '#0ea5e9',
  chain_of_verification:  '#19838e',
  plan_execute:           '#8b9b29',
  oneshot:                '#6b7280',
}
window.patternColor = function(pattern) {
  return PATTERN_COLORS[pattern] || '#9ca3af'
}

Alpine.magic('prettyJson', () => window.prettyJson)
Alpine.magic('shortArgs', () => window.shortArgs)
Alpine.magic('truncate', () => window.truncate)
Alpine.magic('patternColor', () => window.patternColor)

Alpine.start()

// ── helpers ───────────────────────────────────────────────────────────────────

function groupKey(group, depth) {
  return group.id
}
