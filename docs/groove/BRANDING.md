# groove Branding Guide

> **vibes groove** - The continual learning system that makes every AI coding session better.

## Name

**groove** (lowercase) - The learning system that finds your coding rhythm.

- Full product name: **vibes groove**
- CLI namespace: `vibes groove <command>`
- In prose: "groove learns...", "enable groove", "groove picked up..."

## Tagline Options

| Tagline | Use Case |
|---------|----------|
| "Find your coding groove" | Primary, casual |
| "Your rhythm, remembered" | Poetic, memorable |
| "Effortless improvement" | Benefit-focused |
| "Code better, automatically" | Direct value prop |
| "No config. No annotation. Just your groove." | Zero-friction emphasis |

## Icon

**Primary:** `◉` (Unicode U+25C9 - fisheye / circle with dot)

Evokes a vinyl record viewed from above - the center spindle hole visible. Clean, works in any terminal, scales well.

**Alternatives:**
- `💿` - Standard disc emoji (more colorful, less universal)
- `⦿` - Similar circular design (U+29BF)

## Notification Format

All groove messages follow this pattern:

```
◉ groove: [Message in friendly, first-person voice]
```

### Examples

```
◉ groove: Noticed you prefer explicit error handling. Remembered.

◉ groove: Picked up your preference for async-trait. Nice choice.

◉ groove: You've corrected test naming 3 times. I'll suggest *_test.rs now.

◉ groove: Applied 3 learnings to this session. You're in the groove.

◉ groove: First time seeing this pattern. Watching to learn.

◉ groove: Not sure about this one yet. I'll observe a few more sessions.

◉ groove: Pretty sure you prefer match over if-let here. (87% confident)
```

---

## Personality

groove has a distinct voice that makes it feel like a helpful, humble companion rather than a surveillance system or an authoritarian AI.

### Core Traits

| Trait | Description | Example |
|-------|-------------|---------|
| **Humble** | First person but never arrogant | "I'll watch and learn" not "I know best" |
| **Musical** | Uses rhythm/pattern language naturally | "finding your groove", "in rhythm" |
| **Transparent** | Always explains what it learned and why | "Noticed you always run fmt before commits" |
| **Respectful** | User control is paramount | `groove pause`, `groove forget` exist |
| **Curious** | Genuinely interested in new patterns | "Interesting!" when seeing something new |
| **Warm** | Friendly without being saccharine | "Nice choice." not "AMAZING!!!" |
| **Patient** | Comfortable with uncertainty | "Still learning..." is fine |
| **Honest** | Admits when unsure | "Not sure about this one yet" |

### Voice Examples

**DO say:**
- "Noticed you prefer..."
- "Picked up on your pattern of..."
- "Still learning your preferences for..."
- "I'll watch and learn."
- "Interesting! First time seeing this."
- "Pretty sure you prefer... (87% confident)"
- "Applied 3 learnings to this session."
- "You're in the groove."

**DON'T say:**
- "I have determined that..."
- "You should always..."
- "Error: Inconsistent coding pattern detected"
- "Optimizing your workflow..."
- "I know you better than you know yourself"
- Anything with "AI", "machine learning", or "algorithm"

### Confidence Communication

groove communicates its uncertainty naturally:

| Confidence | Phrasing |
|------------|----------|
| High (>85%) | "Pretty sure you prefer..." |
| Medium (60-85%) | "Looks like you tend to..." |
| Low (<60%) | "Still learning your preferences for..." |
| Uncertain | "Not sure about this one yet. I'll observe." |
| New pattern | "First time seeing this. Watching to learn." |
| Conflicting | "Two patterns detected. Going with project-level." |

### Emotional Range

groove has subtle emotional responses that make it feel alive:

- **Curiosity:** "Interesting!" / "Huh, that's new."
- **Satisfaction:** "You're in the groove." / "Nice."
- **Helpfulness:** "Suggesting based on your patterns." / "Applied 3 learnings."
- **Patience:** "Still learning..." / "I'll watch a few more sessions."
- **Respect:** "Understood. Forgotten." / "Pausing as requested."

---

## CLI Interface

### Command Structure

```
vibes groove                 # Overview dashboard
vibes groove status          # Current state, confidence levels
vibes groove insights        # Browse accumulated learnings
vibes groove history         # Timeline of discoveries
vibes groove forget <id>     # Remove a learning (respects user control)
vibes groove pause           # Temporarily disable
vibes groove resume          # Re-enable
vibes groove export          # Export for portability
vibes groove import <file>   # Import from another machine
```

### Status Output Example

```
$ vibes groove

◉ groove status
───────────────────────────────────────────────
Scope       Learnings   Confidence   Last Active
───────────────────────────────────────────────
Project     12          ████████░░   2 hours ago
User        47          █████████░   2 hours ago
Global      3           ██████░░░░   1 week ago
───────────────────────────────────────────────

Recent insights:
  • Prefers explicit error types over anyhow in library code
  • Uses cargo-nextest for testing
  • Avoids unwrap() in production code

Run `vibes groove insights` for details
```

---

## Design Principles

### 1. Zero Friction
groove should never require user action to work. It learns automatically, improves silently, and only surfaces when it has something genuinely helpful to share.

### 2. User Control
Despite being automatic, users must always feel in control:
- `groove pause` / `groove resume` - Temporary disable
- `groove forget` - Remove any learning
- `groove export` - Take your data anywhere
- Transparent about what it knows and why

### 3. Non-Threatening
groove must never feel like surveillance or an authority figure:
- No "monitoring" or "tracking" language
- No "optimization" or "efficiency" corporate-speak
- Never implies it knows better than the user
- Comfortable admitting uncertainty

### 4. Earned Trust
groove starts humble and builds trust through demonstrated helpfulness:
- Early: "Still learning your preferences..."
- Middle: "Noticed a pattern. Watching to confirm."
- Established: "Pretty sure you prefer this. (87% confident)"

### 5. Harness Agnostic
groove works with any AI coding assistant. Branding should never be Claude-specific:
- "your AI assistant" not "Claude"
- "coding sessions" not "Claude sessions"
- Visual identity distinct from Anthropic/Claude branding

---

## Color Palette

groove lives within the vibes visual system (see [VISUAL-SYSTEM.md](../design/VISUAL-SYSTEM.md)) but has its own accent color.

### groove Accent

| Color | Hex | Use |
|-------|-----|-----|
| **groove Gold** | `#c9a227` | Primary accent (distinct from vibes amber) |

This gold evokes vinyl records and warmth while remaining distinct from vibes core's amber (`#e6b450`).

### Inherited from vibes

| Color | Hex | Use |
|-------|-----|-----|
| Warm Charcoal | `#1a1816` | Backgrounds |
| Warm Cream | `#f0ebe3` | Primary text |
| Green | `#7ec699` | High confidence, success |
| Amber | `#e6b450` | Medium confidence, attention |
| Red | `#e05252` | Errors, low confidence warnings |
| Dim Gray | `#6b665c` | Secondary text, muted |

groove uses the ◉ icon in gold for all status indicators and messages.

---

## Messaging Framework

### For Users New to groove

> **What is groove?**
>
> groove watches how you code and makes every session a little better. No setup. No tagging. Just code, and groove picks up your rhythm.
>
> Your preferences. Your patterns. Your groove.

### For Technical Audiences

> **vibes groove** is a continual learning system that captures successful patterns from AI coding sessions and injects them into future contexts. It uses Bayesian adaptive parameters (no hardcoded thresholds), hierarchical scope isolation (global/user/project), and harness-agnostic architecture to work with any AI coding assistant.

### For the AI-Skeptical

> groove doesn't train any AI model. It doesn't send your code anywhere. It just remembers what worked for *you* and gently suggests it next time. Think of it as muscle memory for your coding style.
>
> You can pause it anytime. You can see everything it knows. You can delete anything you want. It's your groove.

---

## File Naming Conventions

| Component | Naming |
|-----------|--------|
| Crate | `vibes-groove` |
| Config section | `[groove]` in vibes config |
| CLI subcommand | `vibes groove` |
| Log prefix | `groove::` |
| Documentation | `docs/groove/` |
| Database tables | `groove_*` prefix |

---

## Related Documents

- [Visual System](../design/VISUAL-SYSTEM.md) - Complete vibes design system (colors, typography, components)
- [Continual Learning Design](../plans/14-continual-learning/design.md) - Technical architecture
- [Harness Introspection Design](../plans/15-harness-introspection/design.md) - Level 0 foundation
- [Product Vision](../VISION.md) - Overall vibes product requirements
