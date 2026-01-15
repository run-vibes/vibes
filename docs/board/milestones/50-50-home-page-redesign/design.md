---
created: 2026-01-14
---

# Milestone 50: Home Page Redesign - Design

> **"Phosphor Mission Control"** — A data-driven dashboard that shows what's happening now and what needs your attention next.

## Overview

The current home page (`/`) is a basic navigation hub with status badges and links to Firehose, Debug, and Sessions. For a system as feature-rich as vibes—with sessions, agents, groove learning, traces, models, and event sourcing—the home page should be a **mission control dashboard** that answers: "What's happening now, and what should I do next?"

### Design Goals

1. **Action-oriented**: "What needs my attention now?" prominently displayed
2. **High information density**: Bloomberg terminal energy for power users
3. **Plugin extensible**: Plugins can contribute dashboard widgets
4. **Real-time**: Live updates via WebSocket for all metrics

### Aesthetic Direction

**Tone**: Industrial-utilitarian meets retro-futuristic. NASA mission control crossed with Bloomberg terminal—dense, glanceable, action-oriented.

**Key visual treatments**:
- Phosphor glow for attention states
- Scanline textures for data areas
- Monospace typography for metrics
- Terminal-inspired status indicators (●/○/◉)

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Layout pattern | Zone-based grid | Separates attention/work/activity/metrics clearly |
| Plugin extensibility | Widget registration API | Allows groove and future plugins to contribute |
| Update strategy | WebSocket push + polling | Real-time for events, polling for aggregates |
| Status indicators | Unicode dots | Consistent with CRT aesthetic, accessible |

## Architecture

### Zone System

The dashboard is organized into four vertical zones:

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                            ATTENTION ZONE                                     │
│  (High-priority action items needing human response)                         │
├──────────────────────────────────────────────────────────────────────────────┤
│                            PRIMARY ZONE                                       │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│  │  Sessions   │ │   Agents    │ │   Groove    │ │  Plugin X   │           │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘           │
├──────────────────────────────────────────────────────────────────────────────┤
│                           SECONDARY ZONE                                      │
│  (Activity stream - full width)                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│                            METRICS ZONE                                       │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐        │
│  │Metric 1│ │Metric 2│ │Metric 3│ │Metric 4│ │Plugin  │ │Plugin  │        │
│  └────────┘ └────────┘ └────────┘ └────────┘ └────────┘ └────────┘        │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Data Sources

| Zone | Data Hook | Update Frequency |
|------|-----------|------------------|
| Attention | `useAttentionItems` (new) | Real-time push |
| Sessions | `useSessionList` | 5s poll |
| Agents | `useAgents` | 5s poll |
| Groove | `useDashboardOverview` | 30s poll |
| Activity | `useFirehose` | Real-time push |
| Metrics | `useSystemMetrics` (new) | 30s poll |

### Plugin Widget API

Plugins can register widgets to appear on the dashboard:

```rust
// In plugin on_load
ctx.register_dashboard_widget(DashboardWidgetSpec {
    id: "groove-pulse".to_string(),
    name: "Groove Pulse".to_string(),
    zone: DashboardZone::Primary,
    priority: 80, // Higher = earlier in layout
    size: WidgetSize::Medium,
});
```

Plugins can also push items to the attention zone:

```rust
ctx.add_attention_item(AttentionItem {
    id: Uuid::new_v7(),
    item_type: AttentionItemType::Plugin,
    priority: 50,
    title: "Groove learning review needed".to_string(),
    description: "3 new patterns extracted".to_string(),
    actions: vec![...],
});
```

---

## Component Specifications

### 1. AttentionBanner

The most critical component. Answers "what needs me right now?"

**Priority order**:
1. Permission requests (agents/sessions awaiting approval)
2. Errors and failures
3. Stalled sessions (no activity > 5 minutes)
4. Groove interventions (circuit breaker triggered)
5. Plugin notifications

**Visual states**:
- **Has items**: Glowing phosphor border, expanded view, pulse animation
- **Empty**: Collapsed to single line "All clear" with green indicator

**Design**:
```
┌────────────────────────────────────────────────────────────────────────────┐
│ ⚡ NEEDS ATTENTION                                                    [⌃⌄] │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│  ◉ PERMISSION                                                     [View →]│
│  Agent "architect" wants to write /src/auth/handler.rs                    │
│  ├─ [Approve] [Deny]                                                      │
│                                                                            │
│  ◉ STALLED                                                        [View →]│
│  Session "refactor-ui" idle for 8 minutes                                 │
│  ├─ [Resume] [End]                                                        │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

### 2. DashboardWidget (Base Component)

Foundation for all primary zone widgets.

**Props**: title, icon, action button, href link, loading state, error state, footer stats

**Design**:
```
┌─────────────────────────────────────────┐
│ TITLE                            [Action]│
│ ─────────────────────────────────────── │
│                                         │
│  (content area)                         │
│                                         │
│  ─────────────────────────────────────  │
│  stat1: val1  │  stat2: val2  │  stat3  │
└─────────────────────────────────────────┘
```

### 3. SessionsWidget

**Row design**:
```
● refactor-ui          processing
  └ "Implementing the new..."  3m ago
```

**Status dots**:
- `●` green = processing (subtle pulse animation)
- `◉` amber = needs attention (permission/error)
- `○` dim gray = idle
- `✕` red = failed

### 4. AgentsWidget

**Row design**:
```
▶ architect-3                 running
  └ Writing auth handler    ████░ 67%
  └ 1,247 tokens  │  12 tool calls
```

**Icons**: `▶` running, `⏸` paused, `⏳` waiting, `✕` failed

### 5. ActivityFeed

**Row design**:
```
14:32:15  agent   architect-3   tool_call    Write /src/auth/handler.rs
```

**Features**:
- Click row to navigate to source
- Auto-scroll with new events (slide-in animation)
- "Load More" button at bottom

### 6. MetricTile

**Variants**:
```
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│ TOKENS/24H  │  │ SUCCESS     │  │ MODELS      │
│ ─────────── │  │ ─────────── │  │ ─────────── │
│   127,432   │  │    94.7%    │  │  2 online   │
│   ↗ +12%    │  │ ████████░░  │  │ claude-4o ● │
└─────────────┘  └─────────────┘  └─────────────┘
  (with trend)    (with progress)  (with status)
```

---

## Keyboard Navigation

| Key | Action |
|-----|--------|
| `s` | Focus sessions widget |
| `a` | Focus agents widget |
| `g` | Navigate to groove dashboard |
| `f` | Navigate to firehose |
| `n` | New session modal |
| `?` | Show keyboard shortcuts modal |

## Responsive Behavior

| Breakpoint | Layout |
|------------|--------|
| Desktop (≥1200px) | 4-column primary grid |
| Laptop (992-1199px) | 3-column primary grid |
| Tablet (768-991px) | 2-column, activity full width |
| Mobile (<768px) | Single column, attention always visible |

## Deliverables

### Design System Components

- [ ] `AttentionBanner` - Action items zone with priority sorting
- [ ] `DashboardWidget` - Base widget component with header/content/footer
- [ ] `ActivityFeed` - Unified event stream with source badges
- [ ] `MetricTile` - Small metric display with sparkline/progress variants

### Web UI Implementation

- [ ] `HomePage` - Replace StreamsPage with zone-based layout
- [ ] `SessionsWidget` - Compact session list widget
- [ ] `AgentsWidget` - Compact agent list widget
- [ ] `useAttentionItems` hook - Aggregate attention items from all sources
- [ ] `useSystemMetrics` hook - Aggregate system-wide metrics

### Plugin API

- [ ] `DashboardWidgetSpec` type in plugin API
- [ ] `register_dashboard_widget` in PluginContext
- [ ] `add_attention_item` in PluginContext
- [ ] `/api/dashboard/widgets` endpoint
- [ ] Dynamic widget loading in frontend

### Groove Integration

- [ ] `GroovePulseWidget` - Plugin-contributed primary widget
- [ ] Attention items for learning reviews
- [ ] Strategy/health metrics tile

---

## Animation Specifications

### Attention Banner Pulse
```css
@keyframes attention-pulse {
  0%, 100% { box-shadow: var(--glow-phosphor); }
  50% { box-shadow: var(--glow-phosphor-bright); }
}
```

### Activity Feed Slide-in
```css
@keyframes slide-in {
  from { transform: translateY(-100%); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}
```

### Status Dot Pulse
```css
@keyframes status-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.6; }
}
```

---

## Open Questions

1. **Widget persistence**: Should users be able to reorder/hide widgets? If so, where is layout stored?

2. **Plugin widget loading**: Dynamic imports or bundle with plugin assets?

3. **Mobile attention**: Should attention items show as a floating notification badge on mobile?
