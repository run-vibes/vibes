# AI-Assisted Verification — Software Requirements Specification

> Use multimodal AI to validate artifacts against story acceptance criteria.

**Epic:** [Coherence Verification](../../PRD.md)
**Status:** backlog

## Scope

Add AI-powered analysis to the verification pipeline. Given a story and its captured artifacts (snapshots, checkpoints, videos), an AI model reviews whether acceptance criteria are met and generates a detailed report with confidence levels.

## Requirements

### Functional Requirements

| ID | Requirement | Source | Verification |
|----|-------------|--------|--------------|
| SRS-01 | Parse acceptance criteria from story markdown files | FR-05 | unit test |
| SRS-02 | Extract `<!-- verify: -->` annotations with optional hints | FR-05 | unit test |
| SRS-03 | Collect referenced artifacts (snapshots, checkpoints, videos) | FR-05 | unit test |
| SRS-04 | Send artifacts to configurable multimodal model for analysis | FR-05 | manual |
| SRS-05 | Support Ollama models (default: qwen3-vl:32b) | FR-05 | manual |
| SRS-06 | Support Claude API as alternative model | FR-05 | manual |
| SRS-07 | Generate verdict (pass/fail/unclear) with confidence level | FR-05 | manual |
| SRS-08 | Produce detailed markdown report with evidence and suggestions | FR-06 | file exists |
| SRS-09 | `just verify ai <story-id>` command runs verification | FR-05 | manual |
| SRS-10 | Model configuration via `verification/config.toml` | FR-05 | file exists |
| SRS-11 | Command-line override for model selection | FR-05 | manual |

### Non-Functional Requirements

| ID | Requirement | Source | Verification |
|----|-------------|--------|--------------|
| SRS-NFR-01 | Graceful degradation if single criterion fails | NFR-03 | manual |
| SRS-NFR-02 | Clear error messages for missing dependencies (Ollama, models) | NFR-03 | manual |
| SRS-NFR-03 | Implementation in TypeScript (consistent with existing tooling) | NFR-03 | code review |

## Stories

| Story | Requirements | Status |
|-------|--------------|--------|
| [CHORE0146](../../../../stages/backlog/stories/[CHORE][0146]-create-ai-verification-config.md) | SRS-10 | backlog |
| [FEAT0197](../../../../stages/backlog/stories/[FEAT][0197]-implement-story-parser.md) | SRS-01, SRS-02 | backlog |
| [FEAT0198](../../../../stages/backlog/stories/[FEAT][0198]-implement-artifact-collector.md) | SRS-03 | backlog |
| [FEAT0199](../../../../stages/backlog/stories/[FEAT][0199]-implement-model-router.md) | SRS-04, SRS-05, SRS-06, SRS-07, SRS-11 | backlog |
| [FEAT0200](../../../../stages/backlog/stories/[FEAT][0200]-implement-ai-report-generator.md) | SRS-08 | backlog |
| [FEAT0201](../../../../stages/backlog/stories/[FEAT][0201]-add-verify-ai-command.md) | SRS-09, SRS-NFR-01, SRS-NFR-02 | backlog |

## Traceability

- **Source:** PRD FR-05 (Automatically detect drift), FR-06 (Generate verification reports)
- **Implements:** Stories (to be created)
- **Verified by:** `just verify ai <story-id>`, manual testing
