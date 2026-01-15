---
id: 54-additional-benchmarks
title: Additional Benchmarks
status: planned
epics: [evals]
---

# Additional Benchmarks

## Overview

Expand benchmark coverage beyond SWE-Bench with HumanEval, Remote Labor Index, and custom benchmark support. Leverages the TaskRunner infrastructure from M53.

## Goals

- Add HumanEval code generation benchmark
- Add Remote Labor Index real-world task benchmark
- Define custom benchmark spec format for user-defined benchmarks
- Enable parallel benchmark task execution
- Provide cross-benchmark comparison tooling

## Key Deliverables

- HumanEval integration — Function completion benchmark
- Remote Labor Index integration — Real-world task completion
- Custom benchmark framework — YAML/TOML spec format
- Benchmark registry — Discover and list available benchmarks
- Parallel execution — Run multiple tasks concurrently
- Comparison tooling — Compare across benchmarks and configurations

## Custom Benchmark Spec

```yaml
name: my-refactoring-benchmark
suite: custom
description: Test refactoring capabilities on real codebases

tasks:
  - id: refactor-001
    description: Extract method from long function
    repo: https://github.com/example/repo
    commit: abc123
    prompt: "Refactor the calculate_totals function to extract tax calculation"
    evaluation:
      type: test_pass
      command: "pytest tests/test_totals.py"

  - id: refactor-002
    description: Rename variable across module
    repo: https://github.com/example/repo
    commit: def456
    prompt: "Rename 'usr' variable to 'user' throughout the auth module"
    evaluation:
      type: diff_match
      expected: patches/refactor-002.patch

config:
  timeout: 300
  runner: docker
  parallelism: 4
```

## Benchmark Types

| Benchmark | Type | Evaluation Method |
|-----------|------|-------------------|
| SWE-Bench | Bug fix | Test suite pass |
| HumanEval | Code generation | Test cases + exact match |
| Remote Labor Index | Task completion | Human evaluation + automated checks |
| Custom | User-defined | Configurable (tests, diff, scripts) |

## Stories

| ID | Title | Priority | Estimate |
|----|-------|----------|----------|
| m54-feat-01 | HumanEval dataset loader | high | 3h |
| m54-feat-02 | HumanEval executor + evaluator | high | 4h |
| m54-feat-03 | Remote Labor Index dataset loader | high | 3h |
| m54-feat-04 | Remote Labor Index executor + evaluator | high | 4h |
| m54-feat-05 | Custom benchmark spec format | medium | 3h |
| m54-feat-06 | Custom benchmark loader | medium | 3h |
| m54-feat-07 | Benchmark registry | medium | 2h |
| m54-feat-08 | Parallel task execution | medium | 3h |
| m54-feat-09 | Cross-benchmark comparison CLI | low | 2h |
| m54-feat-10 | Web UI benchmark comparison | low | 4h |

## Dependencies

- M53 (Benchmark Harness) — TaskRunner infrastructure, Benchmark trait

## Epics

- [evals](../../epics/evals)
