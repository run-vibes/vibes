---
created: 2026-01-14
updated: 2026-01-14
---

# Milestone 50: Home Page Redesign - Design

> **"Phosphor Command"** — A mech suit for generals. An unlimited extension of the resources humans can control.

## Overview

The home page transforms from a navigation hub into a **command interface** for orchestrating AI-powered work at scale. You're not just monitoring — you're steering a force that extends your capabilities without limit.

### Core Mental Model

- **You are the general** — strategic decisions, course corrections, quality judgment
- **Agents are your forces** — executing, discovering, producing
- **The dashboard is your mech suit** — amplifying your awareness and reach
- **Action produces information** — the swarm learns by doing, discoveries emerge from work

### Design Goals

1. **Steering-focused**: "Am I on track? What needs my decision?"
2. **Proof over promises**: Verification artifacts as first-class citizens
3. **Discovery engine**: Surface novel concepts, patterns, and insights
4. **Multiplayer-ready**: Single-player first, scales to teams and autonomous agents
5. **Cost-aware**: Understand economics and optimize for scale

### Aesthetic Direction

**Tone**: Industrial-utilitarian meets retro-futuristic. NASA mission control crossed with military command center — dense, glanceable, action-oriented.

**Key visual treatments**:
- Phosphor glow for attention states
- Scanline textures for data areas
- Monospace typography for metrics
- Terminal-inspired status indicators (●/○/◉)

---

## Command Modes

The interface supports three postures for different commander needs:

| Mode | Posture | What you see |
|------|---------|--------------|
| **Survey** | Glancing check-in | Key metrics, alerts, trajectory |
| **Command** | Active steering | Goals, progress, decisions needed |
| **Deep Dive** | Investigating specifics | Artifacts, research, agent details |

---

## Architecture

### Zone System

The dashboard is organized into seven zones:

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                         COMMAND BAR                                           │
│  [Survey] [Command] [Deep Dive]   Commanders: 👤 👤 🤖 🤖   🔍 ⌘K            │
├──────────────────────────────────────────────────────────────────────────────┤
│                         ATTENTION ZONE                                        │
│  Decisions needed • Anomalies • Verification failures • Course corrections   │
├──────────────────────────────────────────────────────────────────────────────┤
│                         TRAJECTORY ZONE                                       │
│  Goal Progress          │  Cost Trajectory       │  Throughput Trend         │
│  ████████░░ 73%         │  $142/day → $89/day    │  ↗ 23% vs last week       │
├──────────────────────────────────────────────────────────────────────────────┤
│                         PRIMARY ZONE                                          │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐        │
│  │  Sessions    │ │  Agents      │ │  Evaluations │ │  Research    │        │
│  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘        │
├──────────────────────────────────────────────────────────────────────────────┤
│                         DISCOVERY ZONE                                        │
│  Novel Concepts                   │  Coordination Insights                   │
│  💡 Technical discoveries         │  🔗 Emergent patterns                    │
│  🔭 Strategic insights            │  ⚠️ Bottlenecks                          │
├──────────────────────────────────────────────────────────────────────────────┤
│                         ARTIFACTS ZONE                                        │
│  [🎬 Video] [🖼️ Screenshot] [📊 Report] [🎙️ Audio] [📦 Build]                │
├──────────────────────────────────────────────────────────────────────────────┤
│                         METRICS ZONE                                          │
│  Tokens │ Success │ Latency │ Storage │ Compute │ Savings Opportunities      │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Data Sources

| Zone | Data Hook | Update Frequency |
|------|-----------|------------------|
| Attention | `useAttentionItems` | Real-time push |
| Trajectory | `useGoals`, `useCosts` | 30s poll |
| Sessions | `useSessionList` | 5s poll |
| Agents | `useAgents` | 5s poll |
| Evaluations | `useEvaluations` | 30s poll |
| Research | `useResearchQueue` | 30s poll |
| Discovery | `useDiscoveries` | Real-time push |
| Artifacts | `useArtifactStream` | Real-time push |
| Metrics | `useSystemMetrics` | 30s poll |

---

## Verification Artifacts

**Core principle**: Artifacts are proof of work, not just outputs.

```
Agent work → Verification artifacts → Human inspects → Steer/correct
                                              ↓
                                    (if good) → Share with stakeholders
```

### Artifact Types

| Icon | Type | Source |
|------|------|--------|
| 🎬 | Video | Screen recordings, demos, walkthroughs |
| 🖼️ | Image | Screenshots, diagrams, designs |
| 📊 | Report | Generated docs, analyses, summaries |
| 🎙️ | Audio | Podcast generations, voice summaries |
| 📦 | Build | Software artifacts, deployments |
| 📄 | Document | Presentations, specs, plans |

### Artifact Properties

- **Metadata**: Who created, when, which goal/session
- **Verification status**: Passed / failed / pending
- **Actions**: Approve, flag, share, delete

### Infrastructure

Artifacts stored in a **lakehouse architecture**:
- Object storage for cost efficiency at scale
- Apache Arrow for fast analytics
- Multi-modal and unstructured data support
- Queryable across time and projects

---

## Goal Tracking

Goals evolve through maturity levels:

```
Outcome-based (fuzzy)  →  Hierarchical (structured)  →  Metrics-driven (measurable)
"Make onboarding better"   "Reduce steps, add help"      "< 3 min to first value"
```

### Visual States

- 🌱 **Emerging goal** — Outcome-based, still crystallizing
- 🎯 **Structured goal** — Has sub-goals, timeline
- 📊 **Metric-driven** — Clear target, tracking progress

### Goal Widget

```
┌─────────────────────────────────────────────────────────────────┐
│ TRAJECTORY                                            [+ Goal]  │
├─────────────────────────────────────────────────────────────────┤
│  🎯 Ship auth system                              ████████░░ 73%│
│     ├─ Outcome: "Users can log in securely"         ✓ defined  │
│     ├─ Sub-goals: 4/6 complete                      ↗ on track │
│     └─ Target: March 1                              12 days    │
│                                                                 │
│  🌱 Improve onboarding                                    ░░░░░ │
│     └─ Outcome: "New users reach value faster"      ◎ exploring│
└─────────────────────────────────────────────────────────────────┘
```

---

## Discovery Types

The system surfaces discoveries generated by agent work. **"Action produces information."**

| Type | Icon | Generated by |
|------|------|--------------|
| Technical | 💡 | Building, fixing, optimizing |
| Strategic | 🔭 | Research, scanning, market analysis |
| Anomaly | 🔮 | Monitoring, observing patterns |
| Emergent | 🌱 | Agent coordination, self-organization |
| Experimental | 🧪 | A/B tests, trials, experiments |
| Connection | 🔗 | Linking disparate concepts |
| Efficiency | ⚡ | Finding faster/cheaper paths |
| User insight | 👥 | Observing user behavior |
| Risk signal | 🛡️ | Security, reliability, edge cases |

Discovery types are **extensible** — new types emerge as capabilities expand.

---

## Agent Coordination

Three views into swarm coordination:

### Emergent Patterns

Coordination behaviors agents discover on their own:
- Agents can share context via shared memory
- Sequential handoff patterns form naturally
- **"Promote" action**: Codify valuable patterns into explicit mechanisms

### Bottlenecks & Inefficiencies

Where agents are waiting, duplicating, or conflicting:
- Model API queues
- File lock contention
- Resource competition

### Topology Visualization

Full graph view showing:
- Agent nodes and current state
- Communication flows
- Resource dependencies
- Bottleneck highlighting

---

## Research Layers

Research operates in three layers:

| Layer | Mode | Description |
|-------|------|-------------|
| Background | Continuous scanning | Ambient discovery across domains of interest |
| Project | Embedded research | Contextual to active work |
| Focus | Question-driven | Extracting your thoughts into investigations |

### Research Widget

```
┌─────────────────────────────────────────────────────────────────┐
│ RESEARCH                                          [Ask Question]│
├─────────────────────────────────────────────────────────────────┤
│  ACTIVE INVESTIGATIONS                                          │
│  ? "Best approach for JWT refresh tokens"           ██░░ 40%   │
│                                                                 │
│  RECENT FINDINGS                                                │
│  💡 "Redis vs Valkey for session cache"            2h ago      │
│                                                                 │
│  BACKGROUND SCANNING                                            │
│  👁️ Monitoring: auth patterns, rust ecosystem, AI agents       │
└─────────────────────────────────────────────────────────────────┘
```

---

## Cost Intelligence

### Cost Widget

```
┌─────────────────────────────────────────────────────────────────┐
│ COST INTELLIGENCE                              [Full Breakdown] │
├─────────────────────────────────────────────────────────────────┤
│  TODAY        THIS WEEK       PROJECTED MONTH                   │
│  $47.23       $284.12         $892 ±$45                        │
│  ↘ -12%       ↘ -8%           ↘ trending down                  │
│                                                                 │
│  BY RESOURCE                    SAVINGS OPPORTUNITIES           │
│  ████████░░ Compute  $31.40     💡 Switch to Sonnet for evals  │
│  ███░░░░░░░ Storage  $8.20      💡 Batch research queries       │
│  ██░░░░░░░░ Models   $5.80                                      │
└─────────────────────────────────────────────────────────────────┘
```

### Efficiency Metrics

Cost-per-outcome shows **value**, not just spend:
- $/artifact produced
- $/goal completed
- $/research answer

### Projections

- Scenario modeling for scale (2×, 10×, 100× agents)
- Optimization roadmap with potential savings
- Trend visualization with projections

---

## Command Palette (⌘K)

Keyboard-first interface for search, navigation, and actions:

```
┌─────────────────────────────────────────────────────────────────┐
│  ⌘K                                                       [ESC] │
├─────────────────────────────────────────────────────────────────┤
│  > _                                                            │
├─────────────────────────────────────────────────────────────────┤
│  RECENT                                                         │
│  ◆ Goal: Ship auth system                              [g]      │
│  ◆ Session: refactor-ui                                [s]      │
│  ◆ Artifact: demo-video-auth.mp4                       [a]      │
├─────────────────────────────────────────────────────────────────┤
│  ACTIONS                                                        │
│  + New session                                         [n]      │
│  + Ask research question                               [r]      │
│  + Create goal                                         [shift+g]│
├─────────────────────────────────────────────────────────────────┤
│  NEEDS ATTENTION (2)                                            │
│  ◉ Permission: architect wants to write /src/auth      [1]      │
│  ◉ Verification failed: screenshot mismatch            [2]      │
└─────────────────────────────────────────────────────────────────┘
```

### Command Prefixes

| Prefix | Category | Examples |
|--------|----------|----------|
| `g:` | Goals | `g:auth` → jump to auth goal |
| `s:` | Sessions | `s:refactor` → open session |
| `a:` | Artifacts | `a:video` → browse videos |
| `r:` | Research | `r:` → ask a question |
| `?:` | Search all | `?:jwt` → search everything |
| `/` | Commands | `/pause all` `/costs` `/topology` |

---

## Multiplayer Architecture

The system scales from solo commander to distributed command without changing the mental model.

### The Commander Abstraction

A commander can be:
- **Human** — you, teammates, stakeholders
- **AI Agent** — autonomous strategist, specialized lead
- **Robot** — physical world actor with decision authority
- **External System** — CI/CD, monitoring, orchestrators

Commander properties:
- Identity (who/what)
- Authority scope (what can they steer?)
- Presence (online/offline/autonomous)
- Accountability (audit trail of decisions)

### Authority Levels

| Level | Can Steer | Example |
|-------|-----------|---------|
| **Owner** | Everything | Full control |
| **Domain Lead** | Specific goal/area | Sara → auth system |
| **Autonomous Agent** | Delegated domain | Ops-AI → infrastructure |
| **Observer** | Nothing (view only) | Stakeholders, auditors |

### Handoff Modes

| Direction | Description |
|-----------|-------------|
| You → AI | "Take over cost optimization, keep me posted" |
| AI → You | "Hit a decision point I'm not confident about" |
| You → Teammate | "Sara, you own auth now" |
| Shift Change | "End of day, AI takes night shift" |

### Chain of Command

```
                    ┌─────────────┐
                    │   Owner     │
                    └──────┬──────┘
                           │
          ┌────────────────┼────────────────┐
          ▼                ▼                ▼
    ┌───────────┐    ┌───────────┐    ┌───────────┐
    │  Product  │    │Engineering│    │   Ops     │
    │  Lead 👤  │    │  Lead 🤖  │    │  Lead 🤖  │
    └─────┬─────┘    └─────┬─────┘    └─────┬─────┘
          │                │                │
          ▼                ▼                ▼
       Swarms           Swarms           Swarms
```

---

## Component Specifications

### 1. CommandBar

Top-level navigation and presence:
- Mode switching (Survey/Command/Deep Dive)
- Active commanders with presence indicators
- Search and ⌘K trigger
- Quick actions

### 2. AttentionBanner

Priority-sorted action items:
- Permission requests
- Errors and failures
- Stalled sessions
- Verification failures
- Course corrections needed

Visual states: warning (amber glow), critical (red glow), all clear (green)

### 3. TrajectoryWidget

Combined view of goals, costs, and throughput trends:
- Goal progress at different maturity levels
- Cost trajectory with projections
- Throughput trend vs historical

### 4. EvaluationsWidget

Quality signals and verification status:
- Pass/fail status for recent evaluations
- Links to verification artifacts
- Trend over time
- Flaky test detection

### 5. ResearchWidget

Active investigations across all layers:
- Question-driven investigations in progress
- Recent findings
- Background scanning status

### 6. DiscoveryWidget

Novel concepts and coordination insights:
- Discovery cards by type
- Actions: Apply, Save as pattern, Investigate
- Emergent patterns and bottlenecks

### 7. ArtifactsStream

Horizontal scrollable artifact thumbnails:
- Artifact type icons
- Quick preview on hover
- Verification status badge
- Click to inspect

### 8. CoordinationWidget

Factory floor view of agent swarm:
- Emergent patterns (with promote action)
- Bottlenecks and contention
- Throughput efficiency
- Link to full topology view

### 9. CostWidget

Infrastructure economics:
- Daily/weekly/projected costs
- Breakdown by resource type
- Savings opportunities
- Trend sparkline

### 10. MetricTile

Compact metric displays (see original design for variants):
- With trend
- With progress bar
- With sparkline
- With status list

---

## Keyboard Navigation

| Key | Action |
|-----|--------|
| `⌘K` | Open command palette |
| `s` | Focus sessions widget |
| `a` | Focus agents widget |
| `g` | Focus goals/trajectory |
| `d` | Focus discoveries |
| `c` | Focus costs |
| `f` | Navigate to firehose |
| `n` | New session modal |
| `?` | Show keyboard shortcuts |

---

## Responsive Behavior

| Breakpoint | Layout |
|------------|--------|
| Desktop (≥1400px) | Full 4-column primary grid, all zones visible |
| Laptop (1200-1399px) | 3-column primary grid |
| Tablet (768-1199px) | 2-column, stacked zones |
| Mobile (<768px) | Single column, attention always visible, collapsible zones |

---

## Animation Specifications

### Attention Banner Pulse
```css
@keyframes attention-pulse {
  0%, 100% { box-shadow: var(--glow-amber); }
  50% { box-shadow: var(--glow-amber-bright); }
}
```

### Discovery Slide-in
```css
@keyframes discovery-in {
  from { transform: translateX(-20px); opacity: 0; }
  to { transform: translateX(0); opacity: 1; }
}
```

### Commander Presence Pulse
```css
@keyframes presence-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.7; }
}
```

---

## Deliverables

### Design System Components

- [ ] `CommandBar` - Mode switching, presence, search
- [ ] `AttentionBanner` - Action items with priority sorting
- [ ] `TrajectoryWidget` - Goals + costs + throughput
- [ ] `DashboardWidget` - Base widget component
- [ ] `EvaluationsWidget` - Quality signals
- [ ] `ResearchWidget` - Investigation tracking
- [ ] `DiscoveryWidget` - Novel concepts feed
- [ ] `ArtifactsStream` - Horizontal artifact browser
- [ ] `CoordinationWidget` - Agent swarm insights
- [ ] `CostWidget` - Infrastructure economics
- [ ] `CommandPalette` - ⌘K interface
- [ ] `PresenceIndicator` - Commander presence
- [ ] `MetricTile` - Compact metrics (variants)

### Web UI Implementation

- [ ] `HomePage` - Full zone-based layout
- [ ] `useCommanders` hook - Multiplayer presence
- [ ] `useDiscoveries` hook - Discovery feed
- [ ] `useGoals` hook - Goal tracking at all maturity levels
- [ ] `useCosts` hook - Cost tracking and projections
- [ ] `useArtifacts` hook - Artifact stream
- [ ] `useCoordination` hook - Agent swarm insights

### Plugin API

- [ ] `DashboardWidgetSpec` type
- [ ] `register_dashboard_widget` in PluginContext
- [ ] `add_attention_item` in PluginContext
- [ ] `add_discovery` in PluginContext
- [ ] `DiscoveryType` enum (extensible)

### Backend

- [ ] Artifact lakehouse integration
- [ ] Commander/presence system
- [ ] Discovery extraction pipeline
- [ ] Cost aggregation and projection
- [ ] Coordination pattern detection

---

## Open Questions

1. **Discovery persistence**: How long do discoveries stay visible? Archive vs dismiss?

2. **Commander permissions**: Granular permission model for multiplayer?

3. **Artifact retention**: Storage policy for lakehouse at scale?

4. **Autonomous boundaries**: How do AI commanders request authority expansion?

5. **Cross-commander coordination**: Conflict resolution when commanders overlap?
