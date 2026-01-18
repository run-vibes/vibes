# Milestone 33: Groove Dashboard - Design

## Overview

The Groove Dashboard surfaces learning and assessment data for power users, while the system remains invisible by default. It provides observability into the learning system built across milestones 30-32 (extraction, attribution, adaptive strategies).

**Core principle**: Magic happens invisibly by default. Power users can opt into a dashboard showing what's being learned, assessment trends, and attribution insights.

## Navigation Structure

```
/groove                       → Security (existing quarantine)
/groove/assessment/*          → Assessment (existing)
/groove/dashboard             → Overview (new - landing)
/groove/dashboard/learnings   → Learnings (new)
/groove/dashboard/attribution → Attribution (new)
/groove/dashboard/strategy    → Strategy (new)
/groove/dashboard/health      → System Health (new)
```

**Subnav extension:**
```typescript
const grooveSubnavItems = [
  { label: 'Security', href: '/groove', icon: '🛡' },
  { label: 'Assessment', href: '/groove/assessment/status', icon: '◈' },
  { label: 'Dashboard', href: '/groove/dashboard', icon: '📊' },  // NEW
];
```

The Dashboard tab has internal tabs (Overview, Learnings, Attribution, Strategy, Health) following the Assessment layout pattern.

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Navigation | Extend subnav with Dashboard tab | Consistent with Assessment pattern |
| Overview | Hybrid grid with cards | Birds-eye view with drill-down |
| Visualization | Sparklines + full charts | Quick glance + detail on demand |
| Data refresh | WebSocket with initial snapshot | Real-time, efficient updates |
| Learnings page | Split view (list + detail) | Explore without losing context |
| Learning actions | Basic (enable/disable/delete) | Essential actions, advanced in CLI |
| Attribution | Dual view (leaderboard + timeline) | Answer "what helps?" and "why?" |
| System Health | Metrics-focused | Power user info without overwhelm |
| Learning indicator | Opt-in via Settings | Invisible by default, power user opt-in |
| Backend API | WebSocket topics | Per-section subscriptions, partial updates |

---

## Page Designs

### Overview Page (Landing)

Grid of cards providing at-a-glance status with drill-down capability.

```
┌─────────────────────────────────────────────────────────────────────┐
│  DASHBOARD › Overview                                                │
├─────────────────────────────┬───────────────────────────────────────┤
│  SESSION TRENDS             │  LEARNINGS                            │
│  ┌─────────────────────┐    │  Total: 47  Active: 42                │
│  │ ~~~~~~~~▄▄▄▄~~~~    │    │  Recent:                              │
│  │  Score trend        │    │   • Use snake_case (3h ago)           │
│  └─────────────────────┘    │   • Prefer async/await                │
│  ↑ 12% improvement          │                          [View →]     │
│  Last 7 days: 23 sessions   │                                       │
│                  [View →]   │                                       │
├─────────────────────────────┼───────────────────────────────────────┤
│  ATTRIBUTION                │  SYSTEM HEALTH                        │
│  Top Contributors:          │  Assessment: ████████░░ 82%           │
│   1. Error recovery (+0.34) │  Ablation:   ██░░░░░░░░ 18%           │
│   2. snake_case pref (+0.28)│                                       │
│  ⚠ 2 learnings under review │  All systems operational ●            │
│                  [View →]   │                          [View →]     │
└─────────────────────────────┴───────────────────────────────────────┘
```

**Components:**
- `TrendCard` - Sparkline + key metrics + trend direction
- `LearningsCard` - Counts + recent list + category breakdown
- `AttributionCard` - Top contributors + warnings
- `HealthCard` - Progress bars + status indicators

---

### Learnings Page (Split View)

```
┌─────────────────────────────────────────────────────────────────────┐
│  DASHBOARD › Learnings                                               │
├─────────────────────────────────────────────────────────────────────┤
│  [Search...] [Scope ▼] [Category ▼] [Status ▼] [Sort: Value ▼]      │
├────────────────────────────┬────────────────────────────────────────┤
│  LEARNINGS (47)            │  DETAIL                                │
│  ───────────────────────── │  ────────────────────────────────────  │
│  ● Use snake_case for vars │  Use snake_case for variable names    │
│    Project · Correction    │                                        │
│    Value: +0.34 ████████░  │  Category: Correction                  │
│                            │  Scope: Project (vibes)                │
│  ○ Prefer async/await      │  Status: Active ●                      │
│    User · Pattern          │                                        │
│    Value: +0.28 ██████░░░  │  ─── METRICS ───────────────────────   │
│                            │  Estimated Value:  +0.34 (high)        │
│  ○ Test before commit      │  Confidence:       0.87                │
│    Project · Correction    │  Times Injected:   23                  │
│    Value: +0.21 █████░░░░  │  Activation Rate:  78%                 │
│                            │  Sessions:         18                  │
│  ⚠ Avoid console.log       │                                        │
│    Project · Correction    │  ─── SOURCE ────────────────────────   │
│    Value: -0.12 ░░░░░░░░░  │  Extracted: 3 days ago                 │
│    Under review            │  From session: abc-123                 │
│                            │  Method: Correction detection          │
│                            │                                        │
│                            │  ─── ACTIONS ─────────────────────────│
│                            │  [Disable] [Delete]                    │
└────────────────────────────┴────────────────────────────────────────┘
```

**Features:**
- Filterable list with scope, category, status filters
- Sortable by value, confidence, usage, recency
- Detail panel shows full metrics, source info, actions
- Status indicators: Active (●), Disabled (○), Under Review (⚠), Deprecated (✗)
- Enable/Disable and Delete actions with confirmation dialogs

---

### Attribution Page (Dual View)

**Leaderboard tab:**
```
┌─────────────────────────────────────────────────────────────────────┐
│  DASHBOARD › Attribution                                             │
├─────────────────────────────────────────────────────────────────────┤
│  [Leaderboard] [Session Timeline]                     [Last 7 days ▼]│
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  TOP CONTRIBUTORS                    │  NEGATIVE IMPACT              │
│  ─────────────────────────────────── │  ───────────────────────────  │
│  1. Error recovery      +0.34 ●●●●●  │  1. Avoid console.log  -0.12  │
│     87% confidence (23 sessions)     │     Under review              │
│                                      │                               │
│  2. snake_case pref     +0.28 ●●●●○  │  2. Force semicolons   -0.08  │
│     82% confidence (18 sessions)     │     Deprecated                │
│                                      │                               │
│  3. async/await         +0.21 ●●●○○  │                               │
│     74% confidence (12 sessions)     │                               │
│                                      │                               │
│  ─────────────────────────────────────────────────────────────────  │
│  ABLATION COVERAGE                                                   │
│  ████████████████░░░░░░░░░░░░░░░░░░░░░░░░░ 42% of learnings tested  │
│  12 experiments complete · 5 in progress · 30 pending               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

**Session Timeline tab:**
```
┌─────────────────────────────────────────────────────────────────────┐
│  SESSION TIMELINE                                    [Last 7 days ▼] │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Today                                                               │
│  ├─ session-abc │ Score: 0.82 ████████░░ │ 3 learnings activated    │
│  │              │ Error recovery (+0.15), snake_case (+0.08)        │
│  │                                                                   │
│  ├─ session-def │ Score: 0.71 ███████░░░ │ 2 learnings activated    │
│  │              │ async/await (+0.12)                                │
│  │                                                                   │
│  Yesterday                                                           │
│  ├─ session-ghi │ Score: 0.45 ████░░░░░░ │ ⚠ console.log active     │
│  │              │ Negative correlation detected                      │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

### Strategy Page

**Distributions tab:**
```
┌─────────────────────────────────────────────────────────────────────┐
│  DASHBOARD › Strategy                                                │
├─────────────────────────────────────────────────────────────────────┤
│  [Distributions] [Learning Overrides]                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  CATEGORY DISTRIBUTIONS                                              │
│  ─────────────────────────────────────────────────────────────────  │
│                                                                      │
│  Correction + Interactive                      245 sessions          │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │ MainContext     ████████████████████░░░░  0.72              │    │
│  │ Subagent        ████████░░░░░░░░░░░░░░░░  0.31              │    │
│  │ Background      ██░░░░░░░░░░░░░░░░░░░░░░  0.08              │    │
│  │ Deferred        ██████████░░░░░░░░░░░░░░  0.41              │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  ErrorRecovery + Interactive                   156 sessions          │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │ MainContext     ██████████████░░░░░░░░░░  0.58              │    │
│  │ Subagent        ████████████████████░░░░  0.74              │    │
│  │ Background      ████████░░░░░░░░░░░░░░░░  0.32              │    │
│  │ Deferred        ██████░░░░░░░░░░░░░░░░░░  0.25              │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  SPECIALIZED LEARNINGS: 12 of 47 (26%)                              │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

**Learning Overrides tab:**
```
┌─────────────────────────────────────────────────────────────────────┐
│  LEARNING OVERRIDES                              [Specialized only ▼]│
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ● snake_case preference                    47 sessions (specialized)│
│    Base: Correction + Interactive                                    │
│    Override: MainContext 0.81 (vs 0.72 base)                        │
│                                                                      │
│  ● Error retry pattern                      28 sessions (specialized)│
│    Base: ErrorRecovery + Interactive                                 │
│    Override: Subagent 0.89 (vs 0.74 base)                           │
│                                                                      │
│  ○ async/await preference                   12 sessions (inheriting) │
│    Base: Pattern + Interactive                                       │
│    Needs 8 more sessions to specialize                              │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

### Health Page

```
┌─────────────────────────────────────────────────────────────────────┐
│  DASHBOARD › Health                                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  SYSTEM STATUS                                     All systems ● OK  │
│  ─────────────────────────────────────────────────────────────────  │
│                                                                      │
│  ┌─────────────────────┐  ┌─────────────────────┐  ┌──────────────┐ │
│  │  ASSESSMENT         │  │  EXTRACTION         │  │  ATTRIBUTION │ │
│  │  Coverage: 82%      │  │  Active: ●          │  │  Active: ●   │ │
│  │  ████████░░ 82%     │  │  Last: 2m ago       │  │  Last: 5m ago│ │
│  │  23/28 sessions     │  │  47 learnings       │  │  Coverage:   │ │
│  │  Last: 3m ago       │  │  extracted          │  │  42% ablated │ │
│  └─────────────────────┘  └─────────────────────┘  └──────────────┘ │
│                                                                      │
│  ADAPTIVE PARAMETERS                                                 │
│  ─────────────────────────────────────────────────────────────────  │
│                                                                      │
│  │ Parameter              │ Current │ Confidence │ Trend     │      │
│  ├────────────────────────┼─────────┼────────────┼───────────┤      │
│  │ outcome_weight         │ 0.42    │ 0.89       │ stable    │      │
│  │ sentiment_threshold    │ 0.65    │ 0.76       │ ↑ rising  │      │
│  │ ablation_rate          │ 0.10    │ 0.82       │ stable    │      │
│  │ similarity_threshold   │ 0.75    │ 0.71       │ ↓ falling │      │
│  └────────────────────────┴─────────┴────────────┴───────────┘      │
│                                                                      │
│  RECENT ACTIVITY                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│  • 3m ago   Assessment completed for session-xyz (score: 0.78)      │
│  • 5m ago   Attribution updated: snake_case +0.02                   │
│  • 12m ago  Learning extracted: "Prefer early returns"              │
│  • 18m ago  Strategy selected: MainContext for error-recovery       │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

**Status indicators:**
- Green (●) - Operational, processing normally
- Yellow (◐) - Degraded, behind on processing
- Red (○) - Error, needs attention

---

## WebSocket API

### Topics

```rust
/// Dashboard WebSocket subscription topics
pub enum DashboardTopic {
    /// Overview cards - all summary data
    Overview,

    /// Session trends with sparkline data
    Trends { days: u32 },

    /// Learnings list with filters
    Learnings {
        scope: Option<Scope>,
        category: Option<LearningCategory>,
        status: Option<LearningStatus>
    },

    /// Single learning detail
    LearningDetail { id: LearningId },

    /// Attribution leaderboard
    Attribution { days: u32 },

    /// Session timeline for attribution
    SessionTimeline { days: u32 },

    /// Strategy distributions
    StrategyDistributions,

    /// Learning overrides
    StrategyOverrides { specialized_only: bool },

    /// System health metrics
    Health,
}
```

### Messages

```rust
/// Server → Client messages
pub enum DashboardMessage {
    /// Initial snapshot on subscribe
    Snapshot { topic: DashboardTopic, data: DashboardData },

    /// Incremental update
    Update { topic: DashboardTopic, data: DashboardData },

    /// Error response
    Error { topic: DashboardTopic, message: String },
}

/// Client → Server messages
pub enum DashboardRequest {
    Subscribe { topics: Vec<DashboardTopic> },
    Unsubscribe { topics: Vec<DashboardTopic> },

    /// Learning actions
    DisableLearning { id: LearningId },
    EnableLearning { id: LearningId },
    DeleteLearning { id: LearningId },
}
```

**Endpoint:** `/ws/groove/dashboard`

---

## Learning Indicator

### Settings Integration

```
┌─────────────────────────────────────────────────────────────────────┐
│  SETTINGS                                                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ─── GROOVE ─────────────────────────────────────────────────────   │
│                                                                      │
│  Learning Indicator                                    [○ OFF]       │
│  Show 🧠 in header when groove is actively learning                  │
│                                                                      │
│  Dashboard Auto-Refresh                                [● ON]        │
│  Receive real-time updates on dashboard pages                        │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Header Display

When enabled, shows in header next to settings icon:
- Hidden (default) - User hasn't enabled
- Idle (🧠 static) - Enabled but no active processing
- Active (🧠 pulsing) - Currently extracting or attributing
- Error (🧠 red) - Processing error, click for details

---

## Data Models

### Overview Data

```rust
pub struct OverviewData {
    pub trends: TrendSummary,
    pub learnings: LearningSummary,
    pub attribution: AttributionSummary,
    pub health: HealthSummary,
}

pub struct TrendSummary {
    pub sparkline_data: Vec<f64>,
    pub improvement_percent: f64,
    pub trend_direction: TrendDirection,
    pub session_count: u32,
    pub period_days: u32,
}

pub struct LearningSummary {
    pub total: u32,
    pub active: u32,
    pub recent: Vec<LearningBrief>,
    pub by_category: HashMap<LearningCategory, u32>,
}

pub struct AttributionSummary {
    pub top_contributors: Vec<ContributorBrief>,
    pub under_review_count: u32,
    pub negative_count: u32,
}

pub struct HealthSummary {
    pub overall_status: SystemStatus,
    pub assessment_coverage: f64,
    pub ablation_coverage: f64,
    pub last_activity: DateTime<Utc>,
}
```

### Charts

Using visx (already in project) for visualizations:
- `Sparkline` - Compact inline trend charts
- `BarChart` - Strategy distribution weights
- `LineChart` - Full session trend charts
- `ProgressBar` - Coverage and confidence metrics

---

## Future: Diagnostic Mode

Noted for future milestone - expanded System Health with:
- Detailed adaptive parameter history
- Capability gap analysis
- Novelty cluster browser
- Full event replay controls
