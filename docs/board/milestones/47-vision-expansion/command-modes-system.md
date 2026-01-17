# Command Modes System

> **Spoke is your vessel for exploring the universe.** Different expeditions require different configurations.

## The Metaphor

You are an explorer. Spoke is your vessel — a ship configured for traversing the knowledge universe. Just as a real spacecraft has different operational modes (cruise, landing, EVA), your vessel adapts to your current posture.

**The three modes aren't UI tabs. They're vessel configurations.**

| Mode | Posture | You're asking... | Vessel State |
|------|---------|------------------|--------------|
| **Survey** | Glancing, ambient | "Is everything okay? Anything need me?" | Cruise mode. Minimal cockpit. Stars visible. |
| **Command** | Active, steering | "What's happening? What decisions need making?" | Combat mode. Full instrumentation. Ready to act. |
| **Deep Dive** | Investigating, forensic | "Show me everything. I need to understand." | EVA mode. Magnifying glass. Overalls on. In the machinery. |

---

## Product-Wide Posture

Command Modes are **not** dashboard-only. They're a global posture that affects the entire product.

### What Changes Per Mode

| Aspect | Survey | Command | Deep Dive |
|--------|--------|---------|-----------|
| **Information density** | Sparse, glanceable | Balanced, actionable | Dense, comprehensive |
| **Visual aesthetic** | Leans cosmic | Luxury baseline | Leans mechanical |
| **Default actions** | Dismiss, acknowledge | Decide, delegate | Expand, trace, inspect |
| **Keyboard shortcuts** | Navigation-focused | Action-focused | Inspection-focused |
| **Notifications** | Critical only | Actionable items | Everything |
| **Time horizon** | "Right now" | "Today/this week" | "Historical + now" |

### Example: Sessions Page

**Survey Mode:**
- Shows count of active sessions
- Red/yellow/green health indicator
- "3 sessions active. All healthy." — done, glance complete

**Command Mode:**
- List of sessions with status, duration, current task
- Action buttons: pause, resume, terminate
- Attention items highlighted

**Deep Dive Mode:**
- Full session timeline
- Event stream visible
- Token counts, cost breakdown
- Agent decision traces
- Expandable log panels

---

## Connection to Visual Depth

Command Modes interact with the visual depth system:

```
                    SURVEY          COMMAND         DEEP DIVE
                    ───────         ───────         ─────────
Cosmic              ████████        ███░░░░░        ░░░░░░░░
Luxury              ███░░░░░        ████████        ███░░░░░
Mechanical          ░░░░░░░░        ███░░░░░        ████████
Subatomic           ░░░░░░░░        ░░░░░░░░        ███░░░░░
```

- **Survey** pulls the aesthetic toward cosmic — vast, minimal, contemplative
- **Command** centers on luxury — warm, actionable, commander's chair
- **Deep Dive** pulls toward mechanical — dense, technical, engineer's workbench

The mode acts as a **bias** on the visual depth, not an override. A schema browser in Survey mode is still more mechanical than the dashboard in Survey mode, but less mechanical than the same schema browser in Deep Dive mode.

---

## Mode Switching

### Explicit Switching
- Keyboard shortcut: `1` / `2` / `3` or `S` / `C` / `D`
- Mode selector in header (always visible)
- Cmd+K command: `mode survey`, `mode command`, `mode deep`

### Implicit Switching (Future)
The system could detect posture from behavior:
- Rapid navigation → likely Survey
- Hovering, expanding panels → likely Deep Dive
- Taking actions, making decisions → likely Command

**Open question:** Should the system suggest mode switches? "You seem to be investigating. Switch to Deep Dive?"

---

## Vessel Customization (Accessibility & Personalization)

> A disabled user doesn't have a "lesser" experience — they have a vessel configured for their needs, just like any other explorer.

### The Principle

Every explorer configures their ship. Accessibility isn't accommodation — it's **vessel customization**. The machinery adapts to the pilot.

### Customization Dimensions

| Dimension | Examples |
|-----------|----------|
| **Visual** | High contrast, reduced motion, larger text, color blind modes |
| **Motor** | Keyboard-only, switch access, voice control, dwell clicking |
| **Cognitive** | Simplified layouts, reduced information density, guided flows |
| **Sensory** | Screen reader optimization, haptic feedback, audio cues |

### How It Interacts with Modes

Accessibility settings are **orthogonal** to Command Modes. You can be in Deep Dive mode with high contrast and reduced motion — you still get the dense information, just rendered for your needs.

```
Your Vessel = Base Ship + Command Mode + Personal Customizations
```

### Future: Vessel Profiles

Users could save named vessel configurations:
- "Focus Mode" — Deep Dive + notifications off + dark theme
- "Presentation Mode" — Survey + large text + simplified layout
- "Night Shift" — Command + high contrast + reduced motion

---

## Implementation Considerations

### State Management
- Mode is global state, persisted per user
- Mode preference could be per-workspace or global
- URL could encode mode: `?mode=deep` for shareable links

### Progressive Disclosure
Each mode should feel complete, not like you're missing something:
- Survey isn't "Command with stuff hidden" — it's a deliberate minimal view
- Deep Dive isn't "Command with stuff added" — it's a different information architecture

### Performance
- Survey mode should be fastest (less to render)
- Deep Dive may lazy-load panels and traces
- Mode switch should feel instant (skeleton states, not spinners)

---

## Open Questions

1. **Mode memory per page?** Should the system remember "last time you were on Sessions, you were in Deep Dive"?

2. **Mode in multiplayer?** If Sara is in Command and you're in Survey, do you see different things for the same data?

3. **Mode-specific features?** Are there features that only exist in certain modes? Or is everything always accessible, just more/less prominent?

4. **Onboarding?** How do new users learn about modes? Default to Command and let them discover Survey/Deep Dive?

5. **Mobile?** How do modes work on mobile where screen real estate is limited? Does Survey become the default?

6. **Mode indicators?** How do you know what mode you're in? Subtle background shift? Explicit badge? Border color?

---

## Visual Mockup Concepts

### Survey Mode Header
```
┌─────────────────────────────────────────────────────────────────┐
│  ◇ SPOKE                    [Survey ▾]           ⌘K    👤      │
│─────────────────────────────────────────────────────────────────│
│                                                                 │
│              ✓ All systems nominal                              │
│                                                                 │
│              Sessions: 3 active                                 │
│              Agents: 12 running                                 │
│              Cost today: $47                                    │
│                                                                 │
│              [Nothing needs attention]                          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Command Mode Header
```
┌─────────────────────────────────────────────────────────────────┐
│  ◇ SPOKE                    [Command ▾]          ⌘K    👤      │
│─────────────────────────────────────────────────────────────────│
│  ⚡ ATTENTION (2)                                               │
│  ├─ Agent requesting write access to /src/auth        [Review] │
│  └─ Verification failed: screenshot mismatch          [Inspect]│
│─────────────────────────────────────────────────────────────────│
│  SESSIONS          AGENTS           GOALS           COST       │
│  ┌──────────┐      ┌──────────┐    ┌──────────┐   ┌──────────┐│
│  │ 3 active │      │ 12 run   │    │ 73% ship │   │ $47 today││
│  │ 1 stalled│      │ 2 idle   │    │ auth sys │   │ ↘ -12%   ││
│  └──────────┘      └──────────┘    └──────────┘   └──────────┘│
└─────────────────────────────────────────────────────────────────┘
```

### Deep Dive Mode Header
```
┌─────────────────────────────────────────────────────────────────┐
│  ◇ SPOKE        [Deep Dive ▾]   Path: / > Sessions > architect │
│─────────────────────────────────────────────────────────────────│
│  SESSION: architect-refactor-ui                    [Terminate] │
│  Status: Active | Duration: 2h 34m | Tokens: 142K | Cost: $3.21│
│─────────────────────────────────────────────────────────────────│
│  TIMELINE                          │ EVENTS (live)             │
│  ├─ 14:02 Started                  │ 14:36:02 Read file...     │
│  ├─ 14:15 Read 23 files            │ 14:36:04 Tool call...     │
│  ├─ 14:28 First edit               │ 14:36:05 Response...      │
│  ├─ 14:35 Permission request       │ 14:36:08 Edit file...     │
│  └─ 14:36 Waiting for approval     │ ...                       │
└─────────────────────────────────────────────────────────────────┘
```

---

## References

- Visual depth system: [visual-depth-system.md](visual-depth-system.md)
- Dashboard design: [design.md](design.md)
- Prototype: `01-full-dashboard.html` (Command mode baseline)
