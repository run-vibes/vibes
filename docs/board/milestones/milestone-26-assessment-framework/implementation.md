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

| # | Story | Description |
|---|-------|-------------|
| 1 | [chore-01-eventbus-cleanup](stories/chore-01-eventbus-cleanup.md) | Remove dead EventBus code from vibes-core |
| 2 | [feat-02-processor-wiring](stories/feat-02-processor-wiring.md) | Wire assessment components in processor |
| 3 | [chore-03-integration-testing](stories/chore-03-integration-testing.md) | Validate full pipeline end-to-end |
| 4 | [fix-04-plugin-route-mounting](stories/fix-04-plugin-route-mounting.md) | Fix plugin API routes returning HTML |
| 5 | [fix-05-event-flow-to-firehose](stories/fix-05-event-flow-to-firehose.md) | Fix events not flowing to firehose |
| 7 | [refactor-07-plugin-lifecycle](stories/refactor-07-plugin-lifecycle.md) | Add Plugin `on_ready()` lifecycle method |
| 8 | [fix-08-assessment-multiselect](stories/fix-08-assessment-multiselect.md) | Fix assessment page multi-select bug |
| 9 | [feat-09-complete-hook-support](stories/feat-09-complete-hook-support.md) | Support all Claude Code hooks |
| 10 | [feat-10-cli-assess-queries](stories/feat-10-cli-assess-queries.md) | Wire CLI assess commands to query real data |

> **Status:** Check story frontmatter or run `just board` for current status.

## Dependencies

- Story 1 can run independently (cleanup)
- Story 2 is the main integration work
- Story 3 depends on Story 2
- Story 4 is independent (blocks web UI testing)
- Story 5 is independent (blocks E2E validation)
- Story 7 is independent (enables plugin self-containment)
- Story 8 is independent (UI bug fix)
- Story 9 is independent (hook completeness)

## Completion Criteria

- [ ] No dead EventBus code remains
- [ ] Assessment processor routes events through all components
- [ ] CLI commands show real assessment data
- [ ] Integration tests validate full pipeline
- [ ] Plugin API routes return JSON (not HTML)
- [ ] Events flow from Claude hooks through to firehose
- [ ] `just pre-commit` passes

## Reference

Legacy implementation plans (for historical context):
- [legacy-eventlog-plan.md](reference/legacy-eventlog-plan.md) - Original detailed EventLog migration plan
- [legacy-assessment-plan.md](reference/legacy-assessment-plan.md) - Original detailed assessment logic plan
- [milestone-4.4.2a-design.md](reference/milestone-4.4.2a-design.md) - EventLog design document
- [milestone-4.4.2b-design.md](reference/milestone-4.4.2b-design.md) - Assessment logic design document
