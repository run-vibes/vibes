# Evaluation Framework - Product Requirements

> Measure AI performance against benchmarks and over time

## Problem Statement

Teams need to know if their AI setup is actually performing well. Is vibes+groove competitive with other approaches? Is performance improving or regressing over time? The evals framework provides both point-in-time benchmarks and longitudinal measurement of real-world performance.

## Users

- **Primary**: Teams evaluating vibes effectiveness
- **Secondary**: Researchers comparing AI approaches
- **Tertiary**: Developers optimizing their configurations

## Requirements

### Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-01 | Benchmark mode: run industry-standard evals (SWE-Bench, etc.) | must |
| FR-02 | Longitudinal mode: track performance over days/weeks | must |
| FR-03 | Session and task-level metrics | should |
| FR-04 | Trend analysis and forecasting | should |
| FR-05 | Cost tracking per task and session | should |
| FR-06 | Report generation with insights | should |

### Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-01 | Evals completely separate from groove (measure, don't learn) | must |
| NFR-02 | Minimal impact on normal operation when collecting metrics | should |
| NFR-03 | Data retention configurable for compliance | should |

## Success Criteria

- [ ] Can run SWE-Bench and report comparable scores
- [ ] Longitudinal studies show performance trends over weeks
- [ ] Cost tracking provides accurate per-task attribution
- [ ] Reports highlight actionable insights

## Milestones

| ID | Milestone | Status |
|----|-----------|--------|
| 39 | [Performance Evaluation](milestones/39-performance-evaluation/) | in-progress |
| 48 | [Long-term Studies](milestones/48-long-term-studies/) | planned |
| 49 | [Performance Reports](milestones/49-performance-reports/) | planned |
| 50 | [Performance Trends](milestones/50-performance-trends/) | planned |
| 51 | [Benchmark Suite](milestones/51-benchmark-suite/) | planned |
| 52 | [Extended Benchmarks](milestones/52-extended-benchmarks/) | planned |
