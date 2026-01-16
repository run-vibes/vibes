---
id: 49-eval-reports
title: Eval Reports
status: planned
epics: [evals]
---

# Eval Reports

## Overview

Surface longitudinal data through exports, visualizations, and summaries. Delivers immediate value by making collected metrics accessible and actionable.

## Goals

- Generate comprehensive reports from study checkpoints
- Export data in multiple formats (JSON, CSV, Markdown)
- Compute summary statistics across checkpoints
- Compare metrics between time periods
- Provide web UI for exploring eval data

## Key Deliverables

- `ReportGenerator` — Produces `EvalReport` from study checkpoints
- Export formats: JSON, CSV, Markdown with configurable field selection
- Summary statistics: min/max/avg/percentiles for each metric
- Period comparison: this week vs last week, before/after groove changes
- CLI: `vibes eval report` commands
- Web UI: Eval dashboard with study list, checkpoint timeline, metric charts

## Report Structure

```rust
pub struct EvalReport {
    pub study: Study,
    pub period: TimePeriod,
    pub executive_summary: String,
    pub metrics_summary: MetricsSummary,
    pub checkpoints: Vec<Checkpoint>,
    pub comparisons: Vec<PeriodComparison>,
}

pub struct MetricsSummary {
    pub metrics: HashMap<String, MetricStats>,
}

pub struct MetricStats {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub p50: f64,
    pub p95: f64,
}
```

## Stories

| ID | Title | Priority | Estimate |
|----|-------|----------|----------|
| m49-feat-01 | ReportGenerator core | high | 4h |
| m49-feat-02 | JSON/CSV export | high | 2h |
| m49-feat-03 | Markdown report format | medium | 2h |
| m49-feat-04 | Summary statistics | high | 3h |
| m49-feat-05 | Period comparison | medium | 3h |
| m49-feat-06 | CLI report commands | medium | 2h |
| m49-feat-07 | Web UI eval dashboard | medium | 4h |
| m49-feat-08 | Checkpoint timeline chart | low | 3h |

## Dependencies

- M48 (Longitudinal Mode) — Checkpoint data to report on

## Epics

- [evals](../../epics/evals)
