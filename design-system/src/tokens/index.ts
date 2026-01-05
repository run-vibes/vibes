// design-system/src/tokens/index.ts
// CRT Essence Design System - Token Exports
//
// Programmatic access to design tokens for TypeScript consumers

// === Color Tokens ===
export const colors = {
  // Screen (Background) colors
  screen: 'var(--screen)',
  surface: 'var(--surface)',
  surfaceLight: 'var(--surface-light)',
  surfaceLighter: 'var(--surface-lighter)',

  // Phosphor (Primary accent)
  phosphor: 'var(--phosphor)',
  phosphorBright: 'var(--phosphor-bright)',
  phosphorDim: 'var(--phosphor-dim)',
  phosphorFaint: 'var(--phosphor-faint)',
  phosphorGlow: 'var(--phosphor-glow)',
  phosphorSubtle: 'var(--phosphor-subtle)',

  // Text colors
  text: 'var(--text)',
  textDim: 'var(--text-dim)',
  textFaint: 'var(--text-faint)',
  textInverse: 'var(--text-inverse)',

  // Border colors
  border: 'var(--border)',
  borderSubtle: 'var(--border-subtle)',
  borderStrong: 'var(--border-strong)',

  // Semantic colors
  red: 'var(--red)',
  green: 'var(--green)',
  cyan: 'var(--cyan)',
  gold: 'var(--gold)',

  // Scanline
  scanline: 'var(--scanline)',

  // Legacy aliases
  bgBase: 'var(--color-bg-base)',
  bgSurface: 'var(--color-bg-surface)',
  bgElevated: 'var(--color-bg-elevated)',
  bgOverlay: 'var(--color-bg-overlay)',
  textPrimary: 'var(--color-text-primary)',
  textSecondary: 'var(--color-text-secondary)',
  accent: 'var(--color-accent)',
  success: 'var(--color-success)',
  warning: 'var(--color-warning)',
  error: 'var(--color-error)',
  info: 'var(--color-info)',
  groove: 'var(--color-groove)',
  hover: 'var(--color-hover)',
  active: 'var(--color-active)',
  focus: 'var(--color-focus)',
} as const;

// === Typography Tokens ===
export const typography = {
  // Font families
  fontDisplay: 'var(--font-display)',
  fontMono: 'var(--font-mono)',
  fontSans: 'var(--font-sans)',

  // Font sizes
  fontSizeXs: 'var(--font-size-xs)',
  fontSizeSm: 'var(--font-size-sm)',
  fontSizeBase: 'var(--font-size-base)',
  fontSizeLg: 'var(--font-size-lg)',
  fontSizeXl: 'var(--font-size-xl)',
  fontSize2xl: 'var(--font-size-2xl)',
  fontSize3xl: 'var(--font-size-3xl)',

  // Line heights
  leadingNone: 'var(--leading-none)',
  leadingTight: 'var(--leading-tight)',
  leadingNormal: 'var(--leading-normal)',
  leadingRelaxed: 'var(--leading-relaxed)',

  // Letter spacing
  trackingTighter: 'var(--tracking-tighter)',
  trackingTight: 'var(--tracking-tight)',
  trackingNormal: 'var(--tracking-normal)',
  trackingWide: 'var(--tracking-wide)',
  trackingWider: 'var(--tracking-wider)',
  trackingWidest: 'var(--tracking-widest)',

  // Font weights
  fontWeightNormal: 'var(--font-weight-normal)',
  fontWeightMedium: 'var(--font-weight-medium)',
  fontWeightSemibold: 'var(--font-weight-semibold)',
  fontWeightBold: 'var(--font-weight-bold)',

  // Legacy aliases
  textXs: 'var(--text-xs)',
  textSm: 'var(--text-sm)',
  textBase: 'var(--text-base)',
  textLg: 'var(--text-lg)',
  textXl: 'var(--text-xl)',
  text2xl: 'var(--text-2xl)',
} as const;

// === Spacing Tokens ===
export const spacing = {
  space0: 'var(--space-0)',
  spacePx: 'var(--space-px)',
  space0_5: 'var(--space-0-5)',
  space1: 'var(--space-1)',
  space1_5: 'var(--space-1-5)',
  space2: 'var(--space-2)',
  space2_5: 'var(--space-2-5)',
  space3: 'var(--space-3)',
  space4: 'var(--space-4)',
  space5: 'var(--space-5)',
  space6: 'var(--space-6)',
  space8: 'var(--space-8)',
  space10: 'var(--space-10)',
  space12: 'var(--space-12)',
  space16: 'var(--space-16)',
  space20: 'var(--space-20)',
  space24: 'var(--space-24)',
} as const;

// === Border Radius Tokens ===
export const radius = {
  none: 'var(--radius-none)',
  sm: 'var(--radius-sm)',
  md: 'var(--radius-md)',
  lg: 'var(--radius-lg)',
  xl: 'var(--radius-xl)',
  full: 'var(--radius-full)',
} as const;

// === Border Width Tokens ===
export const borderWidth = {
  border0: 'var(--border-0)',
  border1: 'var(--border-1)',
  border2: 'var(--border-2)',
  border3: 'var(--border-3)',
  border4: 'var(--border-4)',
} as const;

// === Transition Tokens ===
export const transitions = {
  fast: 'var(--transition-fast)',
  normal: 'var(--transition-normal)',
  slow: 'var(--transition-slow)',
  slower: 'var(--transition-slower)',
} as const;

// === Z-Index Tokens ===
export const zIndex = {
  base: 'var(--z-base)',
  dropdown: 'var(--z-dropdown)',
  sticky: 'var(--z-sticky)',
  overlay: 'var(--z-overlay)',
  modal: 'var(--z-modal)',
  scanlines: 'var(--z-scanlines)',
  tooltip: 'var(--z-tooltip)',
} as const;

// === Glow Effect Tokens ===
export const glows = {
  phosphor: 'var(--glow-phosphor)',
  phosphorBright: 'var(--glow-phosphor-bright)',
  phosphorDim: 'var(--glow-phosphor-dim)',
  red: 'var(--glow-red)',
  green: 'var(--glow-green)',
  cyan: 'var(--glow-cyan)',
  gold: 'var(--glow-gold)',

  // Legacy aliases
  amber: 'var(--glow-amber)',
  success: 'var(--glow-success)',
  error: 'var(--glow-error)',
} as const;

// === Shadow Tokens ===
export const shadows = {
  sm: 'var(--shadow-sm)',
  md: 'var(--shadow-md)',
  lg: 'var(--shadow-lg)',
  xl: 'var(--shadow-xl)',
  panel: 'var(--shadow-panel)',
  modal: 'var(--shadow-modal)',
  insetSm: 'var(--shadow-inset-sm)',
  insetMd: 'var(--shadow-inset-md)',
} as const;

// === CRT Effect Tokens ===
export const crtEffects = {
  scanlineGradient: 'var(--scanline-gradient)',
  vignetteGradient: 'var(--vignette-gradient)',
  selectionBar: 'var(--selection-bar)',
  selectionBarGlow: 'var(--selection-bar-glow)',
  focusRing: 'var(--focus-ring)',
  focusRingOffset: 'var(--focus-ring-offset)',
} as const;

// === Type Exports ===
export type ColorToken = keyof typeof colors;
export type TypographyToken = keyof typeof typography;
export type SpacingToken = keyof typeof spacing;
export type RadiusToken = keyof typeof radius;
export type BorderWidthToken = keyof typeof borderWidth;
export type TransitionToken = keyof typeof transitions;
export type ZIndexToken = keyof typeof zIndex;
export type GlowToken = keyof typeof glows;
export type ShadowToken = keyof typeof shadows;
export type CrtEffectToken = keyof typeof crtEffects;
