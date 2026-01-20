---
id: 04-performance-trends
title: Performance Trends
status: planned
epics: [evals]
---

# Performance Trends

## Overview

Advanced pattern detection, forecasting, and anomaly identification across longitudinal data. Transforms raw metrics into actionable insights about performance trajectories.

## Goals

- Detect trends in metric time series (improving, stable, declining)
- Forecast future metric values with confidence intervals
- Identify anomalous checkpoints outside expected bounds
- Generate insights correlating groove learnings with metric changes
- Surface recommendations based on detected patterns

## Key Deliverables

- `TrendDetector` — Linear regression over metric time series
- `Forecaster` — Project future values with confidence intervals
- `AnomalyDetector` — Flag outliers using z-score or IQR methods
- `InsightGenerator` — Produce actionable findings with evidence
- Groove correlation analysis — Which learnings correlate with improvements
- CLI: `vibes eval trends` commands
- Web UI: Trends page with interactive charts

## Core Types

```rust
pub struct TrendAnalysis {
    pub metric: String,
    pub trend: Trend,
    pub confidence: f64,
    pub data_points: Vec<(DateTime<Utc>, f64)>,
    pub forecast: Option<Forecast>,
}

pub enum Trend {
    Improving { rate_per_day: f64 },
    Stable { variance: f64 },
    Declining { rate_per_day: f64 },
    Insufficient { min_points_needed: u32 },
}

pub struct Forecast {
    pub horizon_days: u32,
    pub predicted_value: f64,
    pub confidence_interval: (f64, f64),
}

pub struct Insight {
    pub category: InsightCategory,
    pub finding: String,
    pub evidence: Vec<DataPoint>,
    pub recommendation: Option<String>,
}

pub enum InsightCategory {
    Performance,
    Learning,
    Cost,
    Efficiency,
}
```

## Stories

| ID | Title | Priority | Estimate |
|----|-------|----------|----------|
| m50-feat-01 | TrendDetector algorithm | high | 4h |
| m50-feat-02 | Forecaster with confidence intervals | high | 4h |
| m50-feat-03 | Anomaly detector | medium | 3h |
| m50-feat-04 | Insight generator | medium | 4h |
| m50-feat-05 | Groove correlation analysis | high | 4h |
| m50-feat-06 | CLI trends commands | medium | 2h |
| m50-feat-07 | Web UI trends page | medium | 4h |

## Dependencies

- M49 (Eval Reports) — Report infrastructure and summary statistics

## Epics

- [evals](../../epics/evals)
