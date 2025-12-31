// design-system/src/tokens/index.ts
// Token exports for programmatic access

export const colors = {
  // Backgrounds
  bgBase: 'var(--color-bg-base)',
  bgSurface: 'var(--color-bg-surface)',
  bgElevated: 'var(--color-bg-elevated)',
  bgOverlay: 'var(--color-bg-overlay)',
  // Text
  textPrimary: 'var(--color-text-primary)',
  textSecondary: 'var(--color-text-secondary)',
  textDim: 'var(--color-text-dim)',
  textInverse: 'var(--color-text-inverse)',
  // Semantic
  accent: 'var(--color-accent)',
  success: 'var(--color-success)',
  warning: 'var(--color-warning)',
  error: 'var(--color-error)',
  info: 'var(--color-info)',
  // Plugin accents
  groove: 'var(--color-groove)',
  // Borders
  border: 'var(--color-border)',
  borderSubtle: 'var(--color-border-subtle)',
  borderStrong: 'var(--color-border-strong)',
  // Interactive states
  hover: 'var(--color-hover)',
  active: 'var(--color-active)',
  focus: 'var(--color-focus)',
} as const;

export const glows = {
  amber: 'var(--glow-amber)',
  success: 'var(--glow-success)',
  error: 'var(--glow-error)',
} as const;

export const typography = {
  fontMono: 'var(--font-mono)',
  fontSans: 'var(--font-sans)',
  textXs: 'var(--text-xs)',
  textSm: 'var(--text-sm)',
  textBase: 'var(--text-base)',
  textLg: 'var(--text-lg)',
  textXl: 'var(--text-xl)',
  text2xl: 'var(--text-2xl)',
  leadingTight: 'var(--leading-tight)',
  leadingNormal: 'var(--leading-normal)',
  leadingRelaxed: 'var(--leading-relaxed)',
  trackingTight: 'var(--tracking-tight)',
  trackingNormal: 'var(--tracking-normal)',
  trackingWide: 'var(--tracking-wide)',
} as const;

export const spacing = {
  space0: 'var(--space-0)',
  space1: 'var(--space-1)',
  space2: 'var(--space-2)',
  space3: 'var(--space-3)',
  space4: 'var(--space-4)',
  space5: 'var(--space-5)',
  space6: 'var(--space-6)',
  space8: 'var(--space-8)',
  space10: 'var(--space-10)',
  space12: 'var(--space-12)',
  space16: 'var(--space-16)',
} as const;

export const radius = {
  sm: 'var(--radius-sm)',
  md: 'var(--radius-md)',
  lg: 'var(--radius-lg)',
  full: 'var(--radius-full)',
} as const;

export const transitions = {
  fast: 'var(--transition-fast)',
  normal: 'var(--transition-normal)',
  slow: 'var(--transition-slow)',
} as const;

export type ColorToken = keyof typeof colors;
export type GlowToken = keyof typeof glows;
export type TypographyToken = keyof typeof typography;
export type SpacingToken = keyof typeof spacing;
export type RadiusToken = keyof typeof radius;
export type TransitionToken = keyof typeof transitions;
