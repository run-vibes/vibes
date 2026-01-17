# Visual Depth System

> The aesthetic is a depth indicator. You can feel where you are in the system just by the visual language.

## The Metaphor

Spoke uses a physics-inspired visual metaphor: **cosmic** and **subatomic** scales are linked at the extremes. The very large and very small meet — just like in real physics where cosmology and quantum mechanics converge.

```
         ┌─────────────────────────────────────────────────────────────┐
         │                        COSMIC                               │
         │              The highest level — the big picture            │
         │                   Vast. Contemplative. Stars.               │
         └─────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
         ┌─────────────────────────────────────────────────────────────┐
         │                   LUXURY / COMMAND                          │
         │       Dashboard, lakehouse entry, general pages             │
         │              Gold. Warm. Art Deco. Commander.               │
         └─────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
         ┌─────────────────────────────────────────────────────────────┐
         │            MECHANICAL + LUXURY + cosmic hints               │
         │           High-level skills, domain views, catalogs         │
         │          Technical precision creeping in. Warmth fading.    │
         └─────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
         ┌─────────────────────────────────────────────────────────────┐
         │      MECHANICAL + BLUEPRINT + SUBATOMIC + luxury hints      │
         │          Low-level skills, deep technical detail            │
         │         Grid. Annotations. Particles. Quantum feel.         │
         └─────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
                               ┌───────────┐
                               │ SUBATOMIC │ ←────────────┐
                               │  ≈ COSMIC │              │
                               └─────┬─────┘              │
                                     │                    │
                                     └────────────────────┘
                                     (the scales connect)
```

## Level Definitions

### 1. Cosmic

**Where it appears:**
- Skill tree zoomed all the way out
- "Universe" view of all knowledge
- Potentially: high-level product overview/marketing

**Aesthetic:**
- Deep navy-black void
- Scattered colored dots like distant stars/galaxies
- Central golden hub (you) barely visible
- Minimal UI — vast emptiness
- Contemplative, awe-inspiring

**Reference:** `10-skill-tree-v7.html` (zoomed out state)

**Key elements:**
- Background: `var(--bg-void)` with no grid
- Dots: Various colors representing domains (gold, teal, rose, etc.)
- Central hub: Golden glow, "YOUR KNOWLEDGE"
- Feeling: "I am at the center of my knowledge universe"

---

### 2. Luxury / Command

**Where it appears:**
- Main dashboard
- Lakehouse entry/overview
- Sessions list
- Agent management
- Settings
- Any "command center" view

**Aesthetic:**
- Rich gold and navy Art Deco palette
- Warm atmospheric glows
- Elegant typography (Titillium Web, Karla)
- Geometric decorations (chevrons, diamonds, stepped borders)
- Premium materials feeling

**Reference:** `01-full-dashboard.html`, `12-spoke-direction-b-luxurious.html`

**Key elements:**
- Background: Atmospheric gradients, subtle sunburst patterns
- Colors: Gold spectrum (`--gold-primary`, `--gold-bright`)
- Borders: `--border-light`, `--border-gold`
- Glows: `--glow-gold`, `--shadow-gold`
- Feeling: "I am in command. This is my mission control."

---

### 3. Mechanical + Luxury (Transitional)

**Where it appears:**
- High-level skill domains
- Data source list/catalog
- Schema overview
- Connection management
- Query history

**Aesthetic:**
- Luxury foundation with technical precision creeping in
- Some grid lines visible
- More condensed typography
- Technical annotations appearing
- Copper/bronze tones mixing with gold
- Cosmic hints in background (distant stars, subtle glow)

**Key elements:**
- Background: Light grid overlay on luxury base
- Colors: Gold + copper/bronze (`--mech-copper`, `--mech-bronze`)
- Typography: More monospace, more uppercase
- Decorations: Measurement lines, subtle annotations
- Feeling: "I'm getting into the details. Still in command, but hands-on."

---

### 4. Mechanical + Blueprint + Subatomic

**Where it appears:**
- Individual skill deep-dive
- Table/column detail view
- Query execution plan
- Connection config/debugging
- Schema inspector
- Performance profiling

**Aesthetic:**
- Blueprint grid background (20px minor, 100px major)
- Technical drawing annotations
- Monospace typography dominant
- Copper/steel color palette
- Particle effects (small dots, quantum feel)
- Luxury hints in key UI elements (gold accents on buttons, key labels)

**Reference:** `12-spoke-direction-a-mechanical.html`

**Key elements:**
- Background: Blueprint grid (`--mech-grid`, `--mech-grid-strong`)
- Colors: Copper, bronze, steel (`--mech-copper`, `--mech-bronze`, `--mech-steel`)
- Typography: Courier New, technical annotations
- Borders: Dashed measurement lines
- Feeling: "I'm deep in the machinery. Engineer mode."

---

### 5. Subatomic (Links to Cosmic)

**Where it appears:**
- Deepest zoom level in skill tree
- Atomic knowledge units
- Individual event inspection
- Byte-level data view

**Aesthetic:**
- Returns to cosmic feeling but at quantum scale
- Particle fields instead of star fields
- Same void background
- Orbiting electrons instead of orbiting planets
- The grid dissolves back into emptiness
- Hints of warmth returning (the cycle continues)

**Key insight:** At the deepest level, the aesthetic loops back to cosmic. The subatomic and cosmic are visually linked — vast emptiness with floating particles/bodies.

---

## The Animating Force: Iggy

The depth levels describe the *aesthetic* journey, but something crucial animates it all: **Iggy**, the event sourcing backbone.

Iggy is the **nervous system** of Spoke. Events flow like neural impulses through every layer. The event log is memory. Projections are specialized brain regions interpreting the same signals.

This introduces a **biological dimension** that exists *across* all depth levels:

| Depth Level | Biological Presence |
|-------------|---------------------|
| Cosmic | Neural networks as star constellations, synaptic connections as light threads |
| Luxury | Subtle heartbeat animations, pulse rhythms, vital sign indicators |
| Mechanical | Wiring diagrams reveal as neural pathways, event flow as synaptic firing |
| Subatomic | DNA helixes, base pairs as event types, gene sequences as learned patterns |

The Groove plugin (continual learning) visualizes its learning as **neural pathway formation** — repeated patterns draw stronger connections, like axons myelinating with use. You can literally *see* the system getting smarter.

At the subatomic level, physics and biology converge. DNA is molecular. The cosmic-subatomic loop gains new meaning: *"We are made of star-stuff."* The elements in our bodies were forged in dying stars. Life is the universe becoming aware of itself.

**For full documentation:** See [biological-layer.md](biological-layer.md)

---

## Application Map

| View | Primary Level | Secondary Hints |
|------|---------------|-----------------|
| Skill tree (max zoom out) | Cosmic | — |
| Skill tree (domain level) | Cosmic | Luxury (hub glow) |
| Dashboard home | Luxury | Cosmic (atmosphere) |
| Lakehouse overview | Luxury | Cosmic (spoke diagram) |
| Sessions list | Luxury | — |
| Agents view | Luxury | Mechanical (status indicators) |
| Data source catalog | Mechanical + Luxury | Cosmic (distant stars) |
| Schema browser | Mechanical + Blueprint | Luxury (gold accents) |
| Query inspector | Mechanical + Blueprint | — |
| Individual skill (zoomed) | Mechanical + Blueprint | Subatomic (particles) |
| Skill tree (max zoom in) | Subatomic | Cosmic (the loop) |

---

## Color Palette by Level

### Cosmic
```css
--bg-void: #07090d;
/* Colored dots for domains */
--cosmic-gold: #e8c45a;
--cosmic-teal: #5da8a8;
--cosmic-rose: #c98a7a;
--cosmic-blue: #6a9fd4;
```

### Luxury
```css
--gold-primary: #d4a84b;
--gold-bright: #e8c45a;
--gold-dim: #8a6520;
--bg-card: #141a26;
--accent-navy: #1a2d45;
```

### Mechanical
```css
--mech-copper: #c4956a;
--mech-bronze: #a87d52;
--mech-steel: #7a8a9a;
--mech-grid: rgba(100, 130, 160, 0.08);
--mech-annotation: #5d7a94;
```

### Subatomic
```css
/* Same as cosmic, but with particle effects */
--particle-glow: rgba(232, 196, 90, 0.6);
--quantum-blur: 2px;
```

---

## Typography by Level

| Level | Display Font | Body Font | Mono Font |
|-------|--------------|-----------|-----------|
| Cosmic | Titillium Web 300 | — | — |
| Luxury | Titillium Web 300-600 | Karla | — |
| Mechanical | Barlow Condensed | Barlow | Courier New |
| Subatomic | — | — | Courier New |

---

## Transitions

The aesthetic should transition smoothly as users navigate between levels:

1. **Zoom-based:** In the skill tree, zooming in/out morphs between cosmic and subatomic
2. **Navigation-based:** Clicking into a detail view fades in the mechanical elements
3. **Mode-based:** Command modes (Survey/Command/Deep Dive) could shift the aesthetic slightly

**Transition timing:** 0.4s - 0.6s ease for major shifts

---

## Icon System

**Important:** Emoji icons do not fit this aesthetic. They feel childish against the Art Deco/technical treatment.

**Solution:** Custom geometric line-art icons (already defined in `shared.css` as `.deco-icon-*`)

Examples:
- `.deco-icon-target` — Diamond/target shape
- `.deco-icon-session` — Terminal rectangle
- `.deco-icon-agent` — Gear/circle
- `.deco-icon-bolt` — Lightning/energy

All icons should be single-color, geometric, and constructed from CSS shapes or simple SVG paths.

---

## Open Questions

1. **Cosmic → Luxury transition:** How do we get from the skill tree's cosmic view to the dashboard's luxury view? Is it a separate navigation, or does the dashboard have cosmic undertones?

2. **Lakehouse depth:** How deep does the lakehouse go? Overview (luxury) → Source detail (mechanical) → Schema (mechanical + blueprint) → Query plan (subatomic)?

3. **Mobile adaptation:** How does the depth system work on smaller screens where visual richness must be reduced?

4. **Performance:** The cosmic and subatomic levels have particle effects. Need to ensure smooth performance.

---

## Aesthetic Influences

### Art Deco Heritage

The visual language draws from two primary Art Deco influences:

**1. Americana (1920s-30s)**
- Chrysler Building geometric precision
- Rockefeller Center grandeur
- Jazz Age optimism and luxury
- Industrial progress celebration

**2. Ancient Egyptian**
- Direct influence: King Tut's tomb discovery (1922) sparked Art Deco's Egyptian revival
- Gold as the dominant accent color
- Sunburst/radiating patterns (sun god Ra)
- Geometric precision and symmetry
- Stepped forms (pyramids, ziggurats)

### Egyptian Metaphor for Skill Acquisition

The skill tree and knowledge system uses **treasure hunting** as its core metaphor:

| Egyptian Concept | Skill Metaphor | Visual Treatment |
|------------------|----------------|------------------|
| Tomb exploration | Diving into knowledge domains | Zooming deeper into skill tree |
| Unearthing artifacts | Discovering new skills | Reveal animations, golden glow |
| Hieroglyphics | Symbolic expertise language | Geometric icons, shorthand notation |
| Scarabs | Small, precious learnings | Orbiting bodies in skill tree |
| Sarcophagus | Core concepts in layers | Nested detail views |
| Cartouche | Skill badges / achievements | Bordered labels, certifications |
| Pyramid | Hierarchical knowledge | Layered depth system |
| The afterlife | Mastery → teaching → legacy | Contributing back, mentorship |

**Key insight:** You're not just "learning" — you're **excavating ancient wisdom**. Each skill discovered is a treasure unearthed. The deeper you go (mechanical → subatomic), the more precious and rare the artifacts.

### Visual Motifs to Explore

Egyptian-inspired elements that fit the Art Deco language:

- **Winged sun disk** — Mastery indicator, achievement crown
- **Ankh** — Life/active skill symbol
- **Eye of Horus** — Observation, monitoring, awareness
- **Lotus** — Growth, emerging skills
- **Scarab** — Small discoveries, atomic learnings
- **Obelisk** — Milestones, major achievements
- **Cartouche border** — Framing important labels/badges

These should be rendered in the **geometric Art Deco style** (clean lines, minimal curves), not literal Egyptian hieroglyphics. The influence is tonal and structural, not illustrative.

---

## References

- **Cosmic:** `10-skill-tree-v7.html` (zoomed out)
- **Luxury:** `01-full-dashboard.html`, `12-spoke-direction-b-luxurious.html`
- **Mechanical:** `12-spoke-direction-a-mechanical.html`
- **Hybrid example:** `12-spoke-direction-c-hybrid.html`
- **Skill tree (investment model):** `10-skill-tree-v11.html`
