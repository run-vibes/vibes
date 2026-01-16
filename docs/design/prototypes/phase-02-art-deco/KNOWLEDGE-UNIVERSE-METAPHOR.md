# Knowledge Universe Metaphor

## Core Insight

> **"Looking outward = who you know. Looking inward = what you know."**

The Knowledge Universe visualization uses **two distinct realms** connected by a portal:

1. **Space Realm** — The social/relational graph (WHO has knowledge)
2. **Skill Realm** — The knowledge structure (WHAT you know)

---

## Critical Design Learning: Two Realms, Not Layers

**These are separate coordinate systems, not overlapping views.**

Early prototypes (V8) tried to layer both realms in the same space with gradual opacity transitions. This failed because:

- The spacing dynamics are fundamentally different
- Space should feel **vast and distant** (entities are other people, corporations)
- Skills should feel **achievable and gamified** (like leveling up in a video game)
- Blending them creates visual confusion where both are awkwardly visible

**The solution**: Treat them as completely separate worlds with a **portal transition** between them.

### The Portal Transition

The transition between realms should feel like **teleportation**, not gradual zooming:

> "Gradual, then all of a sudden."

- **Approaching**: As you zoom toward the sun, anticipation builds
- **Crossing**: At the threshold, a rapid crossfade (not gradual)
- **Arriving**: You're suddenly in a different world with different rules

Implementation: Sharp eased transition (0.15s) at zoom threshold ~1.8x, with optional visual flash effect.

---

## Space Realm: WHO Has Knowledge

When you look **outward** from yourself, you see the cosmic landscape of knowledge holders.

### Your Solar System (Center)

- **Your Sun** — The Art Deco golden orb. Portal to the Skill Realm.
  - Radial gradient corona with pulse animation
  - Geometric inner ring (Art Deco accent)
  - Label: "Your Knowledge" with concept count
- **Orbiting Collaborators** — People in your knowledge universe:
  - **Inner Orbit**: Close collaborators (team, AI pair)
  - **Middle Orbit**: Peers, friends, mentors
  - **Outer Orbit**: Acquaintances, learners you guide
- Orbital paths shown as dashed circles
- Collaborators orbit with different speeds/directions

### Distant Galaxies (Edges)

External knowledge sources — institutions and individuals with public knowledge:

| Galaxy Type | Color | Icon | Examples |
|-------------|-------|------|----------|
| **Corporations** | Blue | Locked | Google, Microsoft, Stripe |
| **Universities** | Amber | Locked | MIT, Stanford, Berkeley |
| **Governments** | Gray-blue | Locked | NASA, CERN, NIH |
| **Influencers** | Magenta | Locked | Karpathy, Fireship, 3Blue1Brown |
| **Nonprofits** | Green | Open | Wikipedia, Linux Foundation |

### Travel & Access

- Most galaxies are **locked** by default (requires achievements)
- Nonprofits are often **open** — freely accessible knowledge
- Travel represents: contributions, certifications, employment, learning

### Space Realm Spacing

Container: ~4000px. Feels vast. Entities are small, distant. Stars drift slowly.

---

## Skill Realm: WHAT You Know

When you pass through the portal (your sun), you enter a different world — your internal knowledge structure.

### Constellation Layout (V6 Style)

Domains arranged around a central hub with connecting lines:

```
                    Technology
                        |
           Science ─────●───── Health
                       /|\
                      / | \
              Business  |  Creative
                        |
                     Finance
```

- **Center Hub**: "Your Knowledge" with discovery count, orbiting particles
- **Domains**: Octagonal frames arranged in constellation
- **Connection Lines**: Golden gradients from center to each domain
- **States**: Mastered (gold), Learning (teal), Frontier (dashed teal)

### Why Constellation, Not Hexagonal Grid?

V6's constellation layout works better than V8's hexagonal grid because:

- **Achievable**: Discrete nodes feel like video game levels to unlock
- **Clear hierarchy**: Center → Domains → Skills → Concepts
- **Spacious**: Room to zoom and pan without overwhelming density
- **Gamified**: Skill counts, "+N new" badges, progress indicators

### Skill Realm Spacing

Container: ~2000px. Tighter than Space, but still explorable. Domains are large, clickable.

---

## The Zoom Journey (Revised)

```
ZOOM      REALM         WHAT YOU SEE
──────────────────────────────────────────────────────────────────
0.3-1.0   Space         Distant galaxies visible at edges
1.0-1.7   Space         Solar system fills view, sun prominent
═══════════════════════════════════════════════════════════════════ ← PORTAL
1.9-3.0   Skill         Full constellation visible
3.0-6.0   Skill         Individual domains, can click to expand
6.0+      Skill         Deep dive into specific domain skills
```

**Portal Zone (1.7-1.9)**: Rapid crossfade, visual flash, realm switch.

---

## Design Principles

### 1. Separate Coordinate Systems

Each realm has its own:
- Container size and spacing
- Visual language and styling
- Legend and UI context
- Interaction patterns

### 2. The Sun as Portal

The Art Deco sun is the visual link between realms:
- In Space: It's your knowledge mass, the center of your universe
- As Portal: Click or zoom to enter the Skill Realm
- The golden styling carries through to the Skill Realm's center hub

### 3. Context-Aware UI

Legend, indicators, and controls adapt to current realm:
- **Space**: Mentor/Peer/Team/AI | Corp/Univ/Gov/Creator/Open
- **Skill**: Mastered/Learning/Frontier/AI Discovered

### 4. Achievable Knowledge

The Skill Realm should feel like a game:
- Clear progress indicators
- Domains you can "level up"
- New discoveries highlighted
- Frontiers to explore

---

## The Vision: The Greatest Learning Tool

This isn't just a visualization — it IS the mental model for how learning works in the system.

> The whole point is to build the greatest learning tool possible.

---

## Artifact-Based Knowledge Graph

### Core Insight: Evidence, Not Events

**The knowledge graph isn't built from explicit learning events — it's reconstructed from evidence scattered across artifacts.**

| Assumption | Reality |
|------------|---------|
| ~~Users explicitly learn in the system → system tracks what they learned~~ | Users create artifacts everywhere → system observes and infers mastery |

This is a fundamental shift: the system doesn't wait for you to declare "I learned Python." It **observes** your Python usage across all connected sources and reconstructs your knowledge graph from evidence.

### Artifact Sources

| Source Type | Examples | What It Reveals |
|-------------|----------|-----------------|
| **Native** | Conversations, code, projects built with the system | Direct evidence of skills in action |
| **Lakehouse** | Historical data, analytics, behavioral patterns | Long-term mastery trends, usage patterns |
| **Integrations** | Email, calendars, PDFs, PowerPoints, Word, Excel, Notion, Slack, GitHub, etc. | Cross-domain evidence of knowledge application |

### Implications for the Visualization

1. **AI Discovery is Fundamental** — Not a feature, but the core mechanism. The system constantly analyzes artifacts to surface:
   - Skills you demonstrated but never explicitly claimed
   - Mastery levels based on artifact complexity and recency
   - Knowledge connections you didn't realize you had

2. **Evidence Attribution** — Skills should show provenance:
   > "Python: Expert — based on 47 GitHub commits, 12 Jupyter notebooks, 3 presentations"

3. **Confidence Levels** — Inferred mastery has varying certainty:
   - High confidence: Recent, complex, repeated evidence
   - Medium confidence: Older or simpler artifacts
   - Low confidence: Tangential or inferred connections

4. **Explainability** — Users can drill into "Why does the system think I know this?" and see the supporting artifact trail

### Design Decisions

**1. Evidence is Backstage, Insights are the Star**

The system doesn't just reflect artifacts back — it **derives unique insights** from evidence:

| What the system has | What the system shows |
|---------------------|----------------------|
| 47 GitHub commits in Python | "Your Python usage has shifted from data analysis to web development over 6 months" |
| 12 projects using asyncio | "Async patterns are a strength — you've applied them consistently across domains" |
| Decorators only in logging/auth | "Your decorator usage is concentrated — opportunity to expand to caching?" |

Evidence is accessible via "How do you know this?" drill-down, but insights are the default view.

**2. Evidence Layer (Accessible at Every Level)**

Click any skill or element → see supporting artifacts. But artifacts support insights, they don't replace them.

**3. Confidence Modeling**

Confidence affects both visual treatment and language:

| Confidence | Visual | Language |
|------------|--------|----------|
| **High** | Solid border, full opacity | Stated as fact: "You're an expert in..." |
| **Medium** | 80% opacity | Observation: "You've demonstrated..." |
| **Low/Inferred** | Dashed border, 60% opacity, sparkle | Question: "It looks like you might know..." |

**4. User Confirmation Through Questions**

Instead of "Confirm? [Yes/No]" buttons, the system asks contextual questions:

> "We noticed you used asyncio heavily in your last 3 projects — would you say async programming is becoming a focus area?"

> "Your React usage spiked this quarter — are you actively investing in frontend?"

This is conversational refinement, not bureaucratic confirmation.

---

## Skill Trees & Learning Paths

When you zoom into a domain, you see **skill trees with unlock paths** — not just what you know, but routes to learn more.

### Path Types

| Path Type | Description |
|-----------|-------------|
| **System Recommended** | AI-suggested optimal learning routes based on your goals |
| **People Paths** | Routes other humans have taken (mentors, peers, experts) |
| **Agent Paths** | Routes AI agents have taken or synthesized |
| **Your Path** | The actual route you're taking, highlighted differently |

### Teaching Through Visualization

This could be a new way of teaching:
- See how experts progressed through a skill domain
- Follow curated paths designed by mentors
- Let AI recommend next steps based on your current knowledge
- Compare your path to successful learners

---

## AI Discovery

AI doesn't just catalog what you know — it **reveals** what you didn't know:

| Discovery Type | Example |
|----------------|---------|
| **Hidden Skills** | "Based on your work, you already know X" |
| **Unknown Possibilities** | "You could learn Y — it connects to what you know" |
| **Frontier Skills** | "Here's the edge of your knowledge" |

### Discovery UI Concepts

- New discoveries should have a "revelation moment"
- Glow, animation, or visual flourish when AI surfaces something
- Different styling for "you have this" vs "you should learn this"

---

## Knowledge Sharing

When you travel to another sun (collaborator), multiple dimensions of knowledge transfer:

### Formal Sharing (Product Users)

If both parties use the system:
- Learnings tracked and transferable
- **Economic**: Paid access to expertise
- **Pro-bono**: Free knowledge sharing
- Owner controls access

### Informal Sharing (Inference)

Knowledge we can't directly track:
- Inferences from public information
- GitHub contributions, publications, talks
- Social signals of expertise

---

## Space Realm: The Expansion Pack

The Space Realm should feel like an **MMO expansion pack** — exciting, mysterious, with promises of what's coming.

- Keep locked galaxies open-ended
- Could be: employment, certification, contributions, curated portals
- The excitement is in the possibility, not full definition

---

## Deep Zoom Levels — The Full Hierarchy

### Revised Mental Model: Investment, Not Unlocking

**Critical Design Decision:** No "locked" skills. Everything is accessible.

Instead of gates, we show:
- **Invested** vs **Not Yet Invested** (brightness/size)
- **Specialized** vs **Generalist** (depth vs breadth trade-offs)
- **Related skills** that create synergies (not prerequisites that block)

This follows the MechWarrior philosophy: you're customizing a loadout, making trade-off choices about where to invest limited learning time. Not unlocking a linear progression.

### The Complete Zoom Hierarchy

```
LEVEL          METAPHOR              WHAT IT REPRESENTS
─────────────────────────────────────────────────────────────
Space Realm    Cosmic view          WHO has knowledge
Skill Realm    Constellation        WHAT domains you know
Domain View    Skill Network        HOW skills connect within a domain
Periodic       Elements             WHAT concepts make up each skill
Subatomic      Memory particles     WHEN/WHERE you actually used it
```

### Level 3: Domain View — Skill Network

When you zoom into a domain (e.g., Technology), you see a **skill network**:

- Skills as interconnected nodes — relationships, not prerequisites
- **Investment** shown by brightness/size — dimmed = "not yet invested"
- **Everything accessible** — no locks, just choices
- **Paths** as suggested routes through the network
- **AI Discovery** highlights overlooked skills

Visual: V6 aesthetic — octagonal nodes in constellation arrangement, golden connections.

### Level 4: Periodic Table — Elements of Knowledge

Each skill is composed of **elements** — fundamental knowledge units:

| Skill | Example Elements |
|-------|------------------|
| Python | Variables, Loops, Functions, Classes, Decorators |
| Machine Learning | Regression, Classification, Neural Nets, Backprop |

Visual: Grid layout like periodic table, Art Deco styled. Each cell shows symbol, name, proficiency (color intensity), category grouping.

### Level 5: Subatomic — Application & Impact

**Critical Reframe:** NOT memories. The deepest level answers forward-looking questions:

> "Where can this be applied? How will it be useful? What impact/outcome do I predict?"

This makes the subatomic level **actionable**, not nostalgic.

| Component | Purpose |
|-----------|---------|
| **Central Nucleus** | The knowledge element, radiating outward |
| **Application Bubbles** | Concrete domains where this applies (Logging, Caching, Auth...) |
| **Impact Levels** | High (gold), Medium (teal), Exploratory (dimmed) |
| **Utility Cards** | HOW it's useful, pattern recognition cues |
| **Impact Predictions** | WHAT outcomes to expect (quantified where possible) |

Visual: Radiant nucleus with beams connecting to application bubbles. Utility and impact cards in corners.

### Transition Design

| From → To | Transition |
|-----------|------------|
| Space → Skill Realm | Portal teleportation (0.15s crossfade) |
| Constellation → Domain | Domain expands, others fade to edges |
| Domain → Periodic | Skill node expands into element grid |
| Periodic → Subatomic | Element cell reveals memory particles |

Each maintains visual continuity through consistent Art Deco styling.

---

## Gamification Philosophy

All mechanics are on the table, but **taste matters**:

- XP bars
- Achievement unlocks
- Skill trees with prerequisites
- Daily streaks
- Mastery levels

**Key principle**: Don't overdo it. Gamification should feel earned, not gimmicky.

---

## Future Considerations

### Space Realm
- **Wormholes**: Special links to distant galaxies (you contributed to a project)
- **Binary Stars**: Close collaborators with significant knowledge overlap
- **Nebulae**: Areas of emerging knowledge, not yet crystallized

### Skill Realm
- **Atomic/Subatomic Levels**: Deeper zoom into concepts, syntax, edge cases
- **Skill Trees**: Unlock paths within domains
- **Knowledge Decay**: Skills that need refreshing

### Portal
- **Sound Design**: Audio cues for realm transition
- **Personalized Threshold**: Based on knowledge density
- **Return Animation**: Different effect when zooming back out

---

## Reference Prototypes

| Version | What It Demonstrates |
|---------|---------------------|
| V6 | Skill constellation layout, domain frames, orbiting particles — **THE VISUAL STANDARD** |
| V7 | Art Deco sun, space realm, distant galaxies, collaborator orbits |
| V9 | Two-realm portal transition, combining V6 skill + V7 space |
| V10 | ~~Skill trees with locks~~ (deprecated — wrong mental model) |
| V11 | **Full zoom hierarchy**: Domain → Periodic Table → Subatomic (Application & Impact) |
| V12 | **Multiplayer exploration**: Presence, collaboration, organizational views |

---

## Multiplayer Knowledge Sharing

### Core Insight: Knowledge is Social

> **"Learning happens between people. The skill tree isn't just YOUR map — it's a shared territory."**

Multiplayer in the Knowledge Universe goes beyond presence indicators. It's about understanding:
- Who else is exploring this knowledge?
- Who can teach me? Who could I teach?
- What are my team's collective capabilities?
- Where are the gaps?

### Relationship Types

Different relationships have different knowledge sharing dynamics:

| Type | Dynamics | Visual Treatment |
|------|----------|------------------|
| **Friend to Friend** | Casual, exploratory, social learning | Warm colors, playful cursors |
| **Coworker to Coworker** | Professional collaboration, skill gaps | Team badges, project overlays |
| **Mentor to Mentee** | Guided learning, path sharing | Trail indicators, waypoints |
| **Company to Company** | Partnership, capability alignment | Org emblems, heatmaps |

### Presence Features

#### Cursor Visibility
See where others are looking in real-time:
- Cursor shows name/avatar on hover
- Trail fades showing exploration path
- "Here" beacon when someone wants attention

#### Follow/Jump Mechanism
| Action | Behavior |
|--------|----------|
| **Follow** | Your view smoothly tracks their exploration |
| **Jump** | Instant teleport to their current location |
| **Anchor** | Return to your last position |

#### Avatar Indicators
At each zoom level, show who's currently viewing:
- Stacked avatars on domains
- Presence dots on skills
- "X people viewing" counts

### Communication Layers

| Layer | Purpose |
|-------|---------|
| **Ephemeral Chat** | Quick messages, tied to location |
| **Voice Presence** | Audio bubbles, proximity-based volume |
| **Video Tiles** | Face-to-face, docked or floating |
| **Reactions** | Quick feedback, emoji bursts at cursor |

### Actions with Knowledge Primitives

What can you DO with shared knowledge?

| Action | Description |
|--------|-------------|
| **Share Path** | "Here's the route I took to learn this" |
| **Request Teaching** | "Can you explain this to me?" |
| **Propose Goal** | "Let's learn this together" |
| **Mark Gap** | "Our team needs someone with this skill" |
| **Recommend** | "You should explore this next" |
| **Compare** | Side-by-side view of two people's knowledge |

### Organizational Capability View

For managers and companies, a bird's eye view:

#### Team Heatmap
Overlay showing skill distribution across a team:
- **Hot zones**: Deep coverage (gold glow)
- **Warm zones**: Some coverage (teal)
- **Cold zones**: Gaps (dim, dashed)

#### Capability Roster
| Skill | Coverage | Who |
|-------|----------|-----|
| Python | 4 experts, 7 intermediate | Alice, Bob, Carol, ... |
| React | 2 experts | Eve, Frank |
| Machine Learning | 1 expert, 3 learning | Grace, ... |
| Security | GAP | — |

#### Gap Analysis
Identify critical skill gaps:
- "No one on the team knows Kubernetes"
- "Sarah is a single point of failure for ML"
- "Recommended: upskill 2 people in Security"

### Non-Human Entities

The knowledge graph includes more than people:

| Entity Type | Role in Knowledge Graph |
|-------------|------------------------|
| **Software/Tools** | What skills does this tool embody? |
| **Repositories** | What knowledge is encoded here? |
| **Documents** | What topics are covered? |
| **AI Agents** | What capabilities does this agent have? |

#### Entity Relationships
- "This repo contains Python + FastAPI + PostgreSQL"
- "This drive has 47 documents about machine learning"
- "Claude has deep knowledge of coding patterns"

### Visual Design Principles

1. **Presence is Ambient** — Cursors and avatars shouldn't overwhelm the knowledge view
2. **Communication is Contextual** — Chat bubbles appear near relevant skills
3. **Actions are Discoverable** — Right-click menus, hover reveals
4. **Org View is Opt-in** — Individual view by default, toggle for team overlay

### Privacy Considerations

| Level | What Others See |
|-------|-----------------|
| **Public** | Your domain-level expertise areas |
| **Team** | Your skill-level investments |
| **Private** | Your specific knowledge elements, evidence |

Users control visibility. Companies can set defaults but not override personal choice.
