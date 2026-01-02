# Milestone 26: Assessment Framework - Implementation Plan

> **For Claude:** Work through stories in order. Use superpowers:executing-plans for each story.

**Goal:** Complete the assessment framework by wiring up components and validating the pipeline.

**Design:** See [design.md](design.md) for architecture decisions.

---

## Current State

The infrastructure is largely complete:
- **vibes-iggy** crate with EventLog/EventConsumer traits
- **EventLog consumers** in vibes-server (websocket, assessment, notification)
- **Assessment components** implemented and tested individually

What remains is cleanup, integration, and validation.

---

## Stories

| # | Story | Description | Status |
|---|-------|-------------|--------|
| 1 | [chore-01-eventbus-cleanup](stories/chore-01-eventbus-cleanup.md) | Remove dead EventBus code from vibes-core | pending |
| 2 | [feat-02-processor-wiring](stories/feat-02-processor-wiring.md) | Wire assessment components in processor | pending |
| 3 | [chore-03-integration-testing](stories/chore-03-integration-testing.md) | Validate full pipeline end-to-end | pending |

## Dependencies

- Story 1 can run independently (cleanup)
- Story 2 is the main integration work
- Story 3 depends on Story 2

## Completion Criteria

- [ ] No dead EventBus code remains
- [ ] Assessment processor routes events through all components
- [ ] CLI commands show real assessment data
- [ ] Integration tests validate full pipeline
- [ ] `just pre-commit` passes

## Reference

Legacy implementation plans (for historical context):
- [legacy-eventlog-plan.md](reference/legacy-eventlog-plan.md) - Original detailed EventLog migration plan
- [legacy-assessment-plan.md](reference/legacy-assessment-plan.md) - Original detailed assessment logic plan
- [milestone-4.4.2a-design.md](reference/milestone-4.4.2a-design.md) - EventLog design document
- [milestone-4.4.2b-design.md](reference/milestone-4.4.2b-design.md) - Assessment logic design document
