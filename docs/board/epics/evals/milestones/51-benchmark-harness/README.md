---
id: 51-benchmark-harness
title: Benchmark Suite
status: planned
epics: [evals]
---

# Benchmark Suite

## Overview

Build benchmark infrastructure with an extensible TaskRunner architecture. SWE-Bench serves as the proving implementation for the harness.

## Goals

- Define generic Benchmark trait for standardized benchmark execution
- Build extensible TaskRunner with pluggable execution backends
- Implement SWE-Bench integration as first benchmark
- Store benchmark results using event-sourced pattern
- Enable A/B testing with configurable model/agent/groove settings

## Key Deliverables

- `Benchmark` trait — Generic interface for running benchmarks
- `TaskRunner` trait — Pluggable execution backends
- `DockerRunner` — Container-based isolation (default)
- `SandboxRunner` — Native OS sandboxing (macOS Seatbelt, Linux Landlock+seccomp)
- SWE-Bench integration — Dataset loader, task executor, patch evaluator
- Event-sourced result storage
- CLI: `vibes eval run swe-bench`, `vibes eval results`

## Architecture

```
vibes eval run swe-bench --split lite
         │
         ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│ SWE-Bench       │────▶│ Task Runner     │────▶│ Patch Evaluator │
│ Dataset Loader  │     │ (pluggable)     │     │ (test harness)  │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                                         │
                                                         ▼
                                               BenchmarkCompleted event
                                                    → Iggy → Turso
```

## TaskRunner Extensibility

```rust
#[async_trait]
pub trait TaskRunner: Send + Sync {
    fn name(&self) -> &str;
    fn capabilities(&self) -> RunnerCapabilities;

    async fn prepare(&self, task: &BenchmarkTask) -> Result<RunnerSession>;
    async fn execute(&self, session: &RunnerSession, command: &str) -> Result<ExecutionResult>;
    async fn cleanup(&self, session: RunnerSession) -> Result<()>;
}

pub struct RunnerCapabilities {
    pub supports_snapshots: bool,
    pub supports_networking: bool,
    pub max_parallelism: usize,
    pub isolation_level: IsolationLevel,
}

pub enum IsolationLevel {
    Process,      // Nix shell
    Container,    // Docker
    VM,           // Firecracker, QEMU
    Remote,       // SSH to remote host
    Orchestrated, // Kubernetes pod
    Sandbox,      // OS-level (Seatbelt/Landlock)
}
```

## Planned Runners

| Runner | Isolation | Use Case |
|--------|-----------|----------|
| `DockerRunner` | Container | Default, good balance of isolation and speed |
| `SandboxRunner` | OS-level | Fast native execution with filesystem/network restrictions |
| `NixRunner` | Process | Fast iteration, reproducible environments |
| `KubernetesRunner` | Orchestrated | Scale-out benchmarking, CI integration |
| `FirecrackerRunner` | VM | Maximum isolation, security-sensitive benchmarks |
| `RemoteRunner` | SSH | Use existing infrastructure, GPU access |

## SandboxRunner Design

Uses native OS sandboxing for fast, lightweight isolation:

- **macOS**: Seatbelt via `sandbox-exec`
- **Linux**: Landlock filesystem restrictions + seccomp syscall filtering

```rust
pub struct SandboxPolicy {
    pub writable_roots: Vec<PathBuf>,
    pub read_only_roots: Vec<PathBuf>,
    pub network: NetworkPolicy,
    pub blocked_syscalls: Vec<String>,
}

pub enum NetworkPolicy {
    Disabled,
    LocalOnly,
    AllowList(Vec<String>),
    Unrestricted,
}
```

## Benchmark Trait

```rust
#[async_trait]
pub trait Benchmark: Send + Sync {
    fn name(&self) -> &str;
    fn suite(&self) -> BenchmarkSuite;

    async fn load_tasks(&self, config: &BenchmarkConfig) -> Result<Vec<BenchmarkTask>>;
    async fn run_task(&self, task: &BenchmarkTask, runner: &TaskRunner) -> Result<TaskResult>;
    async fn evaluate(&self, task: &BenchmarkTask, result: &TaskResult) -> Result<TaskScore>;
}
```

## Stories

| ID | Title | Priority | Estimate |
|----|-------|----------|----------|
| m51-feat-01 | Benchmark trait and types | high | 3h |
| m51-feat-02 | TaskRunner trait | high | 3h |
| m51-feat-03 | DockerRunner implementation | high | 4h |
| m51-feat-04 | SandboxRunner (macOS Seatbelt) | high | 4h |
| m51-feat-05 | SandboxRunner (Linux Landlock+seccomp) | high | 4h |
| m51-feat-06 | SWE-Bench dataset loader | high | 3h |
| m51-feat-07 | SWE-Bench task executor | high | 4h |
| m51-feat-08 | Patch evaluation harness | high | 4h |
| m51-feat-09 | BenchmarkResult storage | medium | 3h |
| m51-feat-10 | CLI run commands | medium | 2h |
| m51-feat-11 | CLI results/compare commands | medium | 2h |
| m51-feat-12 | NixRunner implementation | low | 3h |
| m51-feat-13 | Web UI benchmark results page | low | 4h |

## Dependencies

- M39 (Eval Core) — EvalStorage, study infrastructure

## Epics

- [evals](../../epics/evals)
