# Biological Design Language

> **Life, not diagrams.** The system doesn't display data — it breathes, pulses, and flows.

## Overview

The biological layer is not a separate aesthetic — it's the **animating force** that makes Spoke feel alive. Where the Art Deco visual system provides the structural language (geometry, typography, color), the biological layer provides **motion, breath, and life**.

This document defines the design principles that differentiate biological visualization from geometric representation.

---

## Core Principle: Organic vs Geometric

| Geometric (avoid) | Biological (prefer) |
|-------------------|---------------------|
| Straight lines | Curved paths (bezier, quadratic) |
| Hard edges | Soft, glowing edges |
| Regular shapes | Irregular, morphing shapes |
| Linear motion | Flowing, drifting motion |
| Mechanical timing | Organic easing (ease-in-out, variable) |
| Uniform sizes | Variable, breathing sizes |
| Static borders | Morphing borders |
| Crisp boundaries | Radial gradients fading to transparent |

---

## The Five Biological Primitives

### 1. Cells

Living containers with:
- **Morphing membrane**: Border-radius values that animate between irregular states
- **Inner glow**: Radial gradient from center, fading outward
- **Soft edge**: Border with matching glow, no hard contrast
- **Nucleus**: Inner element that pulses independently

```css
/* Cell membrane that morphs */
border-radius: 47% 53% 52% 48% / 46% 48% 52% 54%;
animation: membrane-morph 8s ease-in-out infinite;

/* Organic glow */
background: radial-gradient(ellipse at 35% 35%,
  rgba(color, 0.6) 0%,
  rgba(color, 0.3) 40%,
  rgba(color, 0.1) 70%,
  transparent 100%
);
```

### 2. Pathways (Axons)

Connections between elements:
- **SVG bezier curves**, never straight lines
- **Glow layer** behind the main stroke (blur + wider stroke)
- **Traveling signals**: Small dots that animate along the path using `offset-path`
- **Variable thickness** to suggest strength/activity

```css
/* Use offset-path for signals traveling along curves */
.signal {
  offset-path: path('M70,95 Q120,60 180,75');
  animation: travel 2.5s ease-in-out infinite;
}
```

### 3. Pulses

Rhythmic expansion/contraction:
- **Heartbeat**: Double-beat pattern (lub-dub), not single pulse
- **Breathing**: Slow, deep rhythm (4-6 seconds)
- **Activity pulse**: Faster, irregular when processing

```css
/* Heartbeat rhythm (double-beat) */
@keyframes heartbeat {
  0% { transform: scale(1); }
  15% { transform: scale(1.15); }  /* lub */
  30% { transform: scale(1); }
  45% { transform: scale(1.1); }   /* dub */
  60%, 100% { transform: scale(1); }
}
```

### 4. Flow

Continuous streams of particles:
- **Blood cells** through vessels
- **Bioluminescent particles** rising
- **Event streams** as flowing elements

Key: particles should have **varied timing, position, and size**.

```css
/* Stagger delays and positions */
.particle:nth-child(1) { animation-delay: 0s; top: 50%; }
.particle:nth-child(2) { animation-delay: 0.8s; top: 40%; }
.particle:nth-child(3) { animation-delay: 1.6s; top: 60%; }
```

### 5. Bioluminescence

Glowing from within:
- Light emanates from the center, not the edge
- Use `filter: drop-shadow()` for organic glow
- Multiple glow layers at different blur radii
- Never uniform — vary opacity and intensity

```css
/* Bioluminescent glow */
.element {
  background: radial-gradient(circle,
    rgba(color, 0.9) 0%,
    rgba(color, 0.4) 50%,
    transparent 100%
  );
  filter: drop-shadow(0 0 10px rgba(color, 0.6));
  box-shadow:
    0 0 20px rgba(color, 0.4),
    0 0 40px rgba(color, 0.2);
}
```

---

## Motion Principles

### Easing

| State | Easing | Duration |
|-------|--------|----------|
| Breathing | `ease-in-out` | 4-6s |
| Heartbeat | `ease-in-out` | 1-1.5s |
| Signal travel | `ease-in-out` | 2-3s |
| Particle drift | `ease-in-out` | 6-10s |
| Membrane morph | `ease-in-out` | 6-10s |

**Never use `linear`** for biological motion. Life doesn't move at constant speed.

### Staggering

Identical elements should never animate in sync:
- Use `animation-delay` to offset each element
- Vary the delay intervals (not uniform spacing)
- Some elements faster, some slower

### Drift

Particles and organelles should **drift**, not move linearly:
- Combine X and Y movement in the same animation
- Use keyframes at 25%, 50%, 75% with varied positions
- Include slight scale variation

```css
@keyframes drift {
  0%, 100% { transform: translate(0, 0); }
  25% { transform: translate(8px, -5px); }
  50% { transform: translate(3px, 8px); }
  75% { transform: translate(-6px, 3px); }
}
```

---

## Color in the Biological Context

| Biological Element | Color | Meaning |
|--------------------|-------|---------|
| Cell membranes | Teal (`#5da8a8`) | Living, healthy tissue |
| Nucleus / Core | Gold (`#d4a84b`) | Central intelligence, life force |
| Blood / Danger | Deep red (`#c75555`) | Vital flow, or injury |
| Signals | Bright gold (`#e8c45a`) | Neural impulses, information |
| Bioluminescence | Teal + Gold | Ethereal, deep-sea life |

### Transparency is Essential

Biological elements are **not opaque**. They have:
- Translucent membranes
- Glowing cores that fade outward
- Overlapping layers that blend

Use `rgba()` extensively. Nothing should be at full opacity.

---

## Applying Biology to UI Components

### Event Stream → Blood Flow

Events are blood cells flowing through a vessel:
- Continuous stream, not discrete list items
- Cells vary in size slightly
- Staggered timing creates natural flow
- The "vessel" has soft edges, slight glow

### System Health → Heartbeat

The dashboard heartbeat:
- Resting: slow, steady rhythm
- Active: faster, stronger beats
- Stressed: erratic, irregular pattern
- Critical: flatline or alarming rhythm

### Learning (Groove) → Neural Pathway Formation

When patterns are recognized:
- New pathways appear faint
- Repeated patterns strengthen (brighter, thicker)
- Unused patterns fade (synaptic pruning)
- Mastered skills become permanent "highways"

### Connections → Axons

Data source connections:
- Curved SVG paths between nodes
- Signals pulse along the axon when active
- Dormant connections dim
- Broken connections show disruption (not error boxes)

---

## What to Avoid

| Avoid | Why | Instead |
|-------|-----|---------|
| Straight lines | Mechanical, not organic | Use bezier curves |
| Hard-edged boxes | Geometric, not cellular | Use morphing borders, radial gradients |
| Uniform timing | Robotic, synchronized | Stagger animations, vary durations |
| Solid colors | Artificial, flat | Use gradients fading to transparent |
| `animation: linear` | No life has constant velocity | Use `ease-in-out` |
| Identical elements | Clones feel artificial | Vary size, timing, position |
| Sharp corners | Industrial, not biological | Use rounded, irregular shapes |

---

## Reference Prototypes

| Prototype | Demonstrates |
|-----------|--------------|
| `17-biological-organic.html` | Living cells, heartbeat, breathing, bioluminescence, blood flow, neural network |

---

## Relationship to Other Visual Layers

The biological layer **animates** the visual depth system:

| Depth Level | Biological Presence |
|-------------|---------------------|
| **Cosmic** | Neural networks as star constellations, synaptic connections as threads of light |
| **Luxury** | Subtle heartbeat in status indicators, pulse rhythms, vital signs |
| **Mechanical** | Wiring diagrams that reveal as neural pathways, event flow as synaptic firing |
| **Subatomic** | DNA helixes, base pairs as event types, quantum-level firing patterns |

The biological layer is **cross-cutting** — it provides the animation language that makes every depth level feel alive.

---

## Summary

> **Biology is not decoration. It's the difference between a dashboard and a living system.**

The key insight: we're not adding "biological-looking graphics" to a geometric interface. We're making the interface **behave** like a living thing. The cells breathe. The signals pulse. The blood flows. The system is alive.
