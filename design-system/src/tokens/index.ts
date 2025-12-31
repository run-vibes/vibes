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

export type ColorToken = keyof typeof colors;
export type GlowToken = keyof typeof glows;
