# Spoke Brand

> **Spoke — Explore. Connect. Amplify.**

## Brand Evolution

Spoke represents the evolution from "vibes" — a shift from ambient feeling to intentional action. You're not just sensing the vibes; you're commanding your vessel.

### The Name

**Spoke** works on multiple levels:

| Meaning | Connection |
|---------|------------|
| **Wheel spoke** | You're the hub, data sources on the rim, spokes connect them |
| **Neural pathway** | Spokes as axons carrying signals through the nervous system |
| **Past tense of speak** | The system spoke to you, told you what you needed to know |
| **Spokesperson** | Your representative in the data universe, acting on your behalf |

### Core Metaphors

1. **The Vessel** — Spoke is your ship for exploring the knowledge universe
2. **The Hub** — You're at the center, everything connects through you
3. **The Nervous System** — Iggy pulses through the spokes, carrying signals
4. **The Mech Suit** — Amplifies your capabilities, you remain in command

---

## Brand Positioning

### Tagline Options

**Primary:**
> **Spoke — Explore. Connect. Amplify.**

**Alternatives:**
> **Spoke — Your hub. Your data. Your command.**

> **Spoke — The wheel that connects everything.**

> **Spoke — Navigate your data universe.**

### Voice & Tone

| Attribute | Description |
|-----------|-------------|
| **Confident** | Spoke knows what it's doing. No hedging. |
| **Elegant** | Art Deco precision, not startup casual |
| **Technical** | Respects the user's intelligence |
| **Warm** | Luxury, not cold enterprise |
| **Exploratory** | Invites discovery, not just utility |

### Personality

Spoke is:
- A skilled navigator, not a backseat driver
- Your capable first officer, not your boss
- Precise but not pedantic
- Powerful but not overwhelming

---

## Visual Identity

### Color Palette

From the Art Deco design system:

```css
/* Primary */
--gold-primary: #d4a84b;
--gold-bright: #e8c45a;
--gold-dim: #8a6520;

/* Background */
--bg-void: #07090d;
--bg-card: #141a26;
--accent-navy: #1a2d45;

/* Accents */
--cosmic-teal: #5da8a8;
--mech-copper: #c4956a;
```

### Typography

| Use | Font | Weight |
|-----|------|--------|
| Display / Logo | Titillium Web | 300 (Light) |
| Headings | Titillium Web | 400-600 |
| Body | Karla | 400 |
| Technical | Courier New / Barlow | 400 |

### Logo

The primary logo is from **Direction B** (`12-spoke-direction-b-luxurious.html`):

**Key elements:**
- **Outer ring** — Gold gradient stroke
- **Inner dashed ring** — Subtle decorative element at radius 20
- **Hub ring** — Stroke ring around the core
- **Hub core** — Solid gold center dot
- **6 spokes** — Arranged at 60° intervals (like a clock), gradient strokes
- **Spokes end at hub ring** — They don't extend to the center

**Why this works:**
- 6 spokes feel more complete than 4 (compass/clock resonance)
- Dashed inner ring adds Art Deco elegance without clutter
- Hub ring + core creates depth (not just a dot)
- Gradient strokes add warmth
- The proportions are refined — nothing touches that shouldn't

**Alternative variations** explored in `13-spoke-logo-prototypes.html`:
- 4-spoke minimal
- 8-spoke compass
- Neural hint (branching ends)
- Sunburst
- Diamond hub
- And more

The primary Direction B logo is preferred over all alternatives.

---

## Domain & URLs

### Primary Domain

```
www.spoke.sh          → Main marketing site (canonical)
spoke.sh              → Redirects to www.spoke.sh
```

### Subdomains

```
app.spoke.sh          → Web UI (dashboard, sessions, agents)
docs.spoke.sh         → Documentation
api.spoke.sh          → API endpoints
cli.spoke.sh          → CLI download and installation
status.spoke.sh       → System status page
```

### The .sh TLD

Works on multiple levels:
- **Shell** — CLI-first, terminal native
- **Ship** — Your vessel (spoke.sh = spoke ship)
- **Short** — Clean, memorable, technical

---

## Messaging Framework

### For Developers

> Spoke is a hybrid execution engine that connects your local environment to cloud data. Query anything — SQL, vectors, graphs — with computation pushed to where the data lives.

### For Data Teams

> Spoke unifies your data sources into a single queryable hub. Federation, not migration. Your data stays where it is; Spoke connects it all.

### For Technical Leaders

> Spoke is the nervous system for your data infrastructure. Event-sourced, version-controlled, with security policies that respect data gravity.

### For the Curious

> Spoke is your vessel for exploring the data universe. Connect to anything, query everything, learn as you go.

---

## Brand Applications

### CLI

```
$ spoke

   ◇ SPOKE — Explore. Connect. Amplify.

   Commands:
     session    Manage sessions
     query      Run queries
     connect    Add data sources
     ...
```

### Web UI Header

```
┌─────────────────────────────────────────────────────────────────┐
│  ◇ SPOKE                    [Command ▾]          ⌘K    👤      │
└─────────────────────────────────────────────────────────────────┘
```

### Documentation

```
┌─────────────────────────────────────────────────────────────────┐
│  ◇ SPOKE DOCS                                                   │
│                                                                 │
│  Getting Started                                                │
│  ├── Installation                                               │
│  ├── Your First Query                                           │
│  └── Connecting Data Sources                                    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Decisions Made

| Decision | Choice |
|----------|--------|
| **Logo** | Direction B original (6 spokes, dashed inner ring, hub ring + core) |
| **Domain** | www.spoke.sh (canonical), spoke.sh redirects |
| **Positioning** | Exploration + Connection + Amplification |

## Open Questions

1. **Wordmark treatment** — "SPOKE" vs "Spoke" vs "spoke"?
2. **Icon vs full logo** — When to use mark alone vs with wordmark?
3. **Color variations** — Gold on dark only? Light mode version?
4. **Animation** — Should the logo animate (spokes rotating, pulsing)?
5. **Favicon** — Does the logo work at 16px, or do we need a simplified version?

---

## References

- [Visual Depth System](visual-depth-system.md) — Aesthetic hierarchy
- [Command Modes](command-modes-system.md) — Vessel metaphor
- [Biological Layer](biological-layer.md) — Neural/nervous system
- [Lakehouse Architecture](lakehouse-architecture.md) — Hub and spoke diagram
- Logo prototypes: `13-spoke-logo-prototypes.html`
