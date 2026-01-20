# AI-Assisted Verification — Design Document

> Configurable multimodal AI analysis of verification artifacts against acceptance criteria.

**SRS:** [SRS.md](SRS.md)

## Overview

AI-Assisted Verification adds intelligent analysis to the existing verification pipeline. Given a story and its captured artifacts, an AI model reviews whether acceptance criteria are met and produces a detailed report with confidence levels.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    AI VERIFICATION FLOW                          │
│                                                                   │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐            │
│  │   Story     │   │  Artifacts  │   │   Model     │            │
│  │  Parser     │──▶│  Collector  │──▶│  Router     │            │
│  │             │   │             │   │             │            │
│  └─────────────┘   └─────────────┘   └─────────────┘            │
│        │                 │                 │                     │
│        ▼                 ▼                 ▼                     │
│  ┌─────────────────────────────────────────────────┐            │
│  │              Verification Engine                 │            │
│  │  • Parse acceptance criteria from story          │            │
│  │  • Match criteria to artifacts via annotations   │            │
│  │  • Route to configured model (Ollama/Claude)     │            │
│  │  • Aggregate results with confidence levels      │            │
│  └─────────────────────────────────────────────────┘            │
│                           │                                      │
│                           ▼                                      │
│                   ┌───────────────┐                              │
│                   │  AI Report    │                              │
│                   │  (markdown)   │                              │
│                   └───────────────┘                              │
└─────────────────────────────────────────────────────────────────┘
```

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Default model** | Ollama qwen3-vl:32b | Local-first, no API costs, good multimodal support |
| **Implementation** | TypeScript | Consistent with existing web-ui tooling |
| **Configuration** | TOML file | Simple, human-readable, matches project style |
| **Confidence levels** | High/Medium/Low | Clear thresholds for trust vs review |
| **Report format** | Markdown | Matches existing verification reports |

## Components

### Story Parser (`verification/scripts/lib/parser.ts`)

Extracts acceptance criteria and verification annotations from story markdown.

```typescript
interface ParsedStory {
  id: string;
  title: string;
  scope: string;
  criteria: Criterion[];
}

interface Criterion {
  text: string;
  annotation?: {
    type: 'snapshot' | 'checkpoint' | 'video';
    name: string;
    hint?: string;  // Optional: "should show 3 sessions"
  };
}
```

### Artifact Collector (`verification/scripts/lib/collector.ts`)

Gathers referenced artifacts from `verification/` directory.

```typescript
interface CollectedArtifact {
  type: 'image' | 'video';
  path: string;
  data: Buffer;  // For images, base64 for API
}
```

### Model Router (`verification/scripts/lib/router.ts`)

Routes artifacts to configured model.

```typescript
interface ModelConfig {
  provider: 'ollama' | 'claude';
  model: string;  // e.g., 'qwen3-vl:32b' or 'claude-sonnet-4-20250514'
}

async function analyze(
  artifact: CollectedArtifact,
  criterion: Criterion,
  config: ModelConfig
): Promise<Verdict>
```

### Report Generator (`verification/scripts/lib/report.ts`)

Produces markdown report from verdicts.

## Configuration

`verification/config.toml`:

```toml
[ai]
# Default model for all artifacts
default_model = "ollama:qwen3-vl:32b"

# Confidence thresholds
[ai.confidence]
high = 80    # ≥80% = trust result
medium = 50  # 50-79% = review recommended
# <50% = human review required
```

## Commands

| Command | Description |
|---------|-------------|
| `just verify ai <story-id>` | Run AI verification for a story |
| `just verify ai <story-id> --model "ollama:llava:34b"` | Override model |

### Just Command

```just
# Run AI verification for a story
ai STORY_ID *ARGS:
    npx tsx verification/scripts/ai-verify.ts {{STORY_ID}} {{ARGS}}
```

## Report Format

Generated at `verification/reports/<scope>/<id>-ai.md`:

```markdown
# AI Verification Report: FEAT0109

**Story:** Board generator grouped layout
**Scope:** coherence-verification/01-artifact-pipeline
**Model:** ollama:qwen3-vl:32b
**Generated:** 2026-01-19 14:32:05

## Summary

| Result | Count |
|--------|-------|
| ✅ Pass | 3 |
| ❌ Fail | 1 |
| ⚠️ Needs Review | 1 |

## Criteria

### 1. Sessions page displays list
**Artifact:** `snapshots/sessions.png`
**Verdict:** ✅ Pass (High confidence: 92%)

> The screenshot shows a sessions page with a table containing 4 session
> rows. Each row displays session ID, status, and timestamp.
```

## Prompt Structure

```typescript
const VERIFICATION_PROMPT = `
You are verifying if a UI artifact meets an acceptance criterion.

**Criterion:** {criterion}

**Additional context:** {annotation_hint}

Analyze the provided artifact and determine:
1. Does this artifact demonstrate the criterion is met?
2. What specific evidence supports your verdict?
3. How confident are you? (0-100%)

Respond in JSON:
{
  "verdict": "pass" | "fail" | "unclear",
  "confidence": <0-100>,
  "evidence": "<what you observed>",
  "suggestion": "<if fail, how to fix>" | null
}
`;
```

## Error Handling

| Scenario | Behavior |
|----------|----------|
| Story not found | Exit with error: `Story FEAT0109 not found` |
| No acceptance criteria | Exit with error: `No acceptance criteria found` |
| No verify annotations | Warning, analyze all criteria against all artifacts |
| Artifact missing | Report shows `⚠️ Artifact not found` |
| Ollama not running | Exit with error: `Ollama not reachable. Run: ollama serve` |
| Model not available | Exit with error: `Model not found. Run: ollama pull <model>` |
| Model timeout | Report shows `⚠️ Analysis timed out` |

## Data Flow

1. User runs `just verify ai FEAT0109`
2. Script finds story file by ID
3. Parser extracts criteria and annotations
4. Collector gathers referenced artifacts
5. Router loads config, determines model
6. For each criterion:
   - Send artifact + criterion to model
   - Receive verdict with confidence
7. Report generator creates markdown
8. Report saved to `verification/reports/<scope>/<id>-ai.md`

## Dependencies

```json
{
  "devDependencies": {
    "ollama": "^0.5.0",
    "@anthropic-ai/sdk": "^0.25.0",
    "toml": "^3.0.0",
    "tsx": "^4.0.0"
  }
}
```

## Deliverables

- [ ] `verification/config.toml` — Model configuration
- [ ] `verification/templates/ai-report.md` — Report template
- [ ] `verification/scripts/ai-verify.ts` — Main entry point
- [ ] `verification/scripts/lib/parser.ts` — Story parser
- [ ] `verification/scripts/lib/collector.ts` — Artifact collector
- [ ] `verification/scripts/lib/router.ts` — Model router
- [ ] `verification/scripts/lib/report.ts` — Report generator
- [ ] `.justfiles/verify.just` — Add `ai` command
- [ ] Documentation in CLAUDE.md
