# Web UI Modernization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Transform vibes web UI to the warm terminal aesthetic with a reusable design system and Iggy stream views.

**Architecture:** Create `@vibes/design-system` workspace package with design tokens, primitives, and compositions. Use Ladle for component development. Add `/ws/firehose` WebSocket endpoint for real-time event streaming. Migrate web-ui to consume the design system.

**Tech Stack:** React 18, TypeScript, Vite, Ladle, CSS Modules, xterm.js, TanStack Router/Query

---

## Phase 1: Design System Foundation

### Task 1.1: Create design-system package

**Files:**
- Create: `design-system/package.json`
- Create: `design-system/tsconfig.json`
- Create: `design-system/vite.config.ts`
- Create: `design-system/src/index.ts`
- Modify: `package.json` (root workspace)

**Step 1: Create package.json**

```json
{
  "name": "@vibes/design-system",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "main": "./src/index.ts",
  "types": "./src/index.ts",
  "exports": {
    ".": "./src/index.ts",
    "./tokens": "./src/tokens/index.css"
  },
  "scripts": {
    "dev": "ladle serve",
    "build": "ladle build",
    "test": "vitest",
    "test:ui": "vitest --ui"
  },
  "dependencies": {
    "react": "^18",
    "react-dom": "^18"
  },
  "devDependencies": {
    "@ladle/react": "^4",
    "@testing-library/react": "^14",
    "@types/react": "^18",
    "@types/react-dom": "^18",
    "@vitejs/plugin-react": "^4",
    "typescript": "^5",
    "vite": "^5",
    "vitest": "^1"
  }
}
```

**Step 2: Create tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true
  },
  "include": ["src"]
}
```

**Step 3: Create vite.config.ts**

```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
});
```

**Step 4: Create src/index.ts**

```typescript
// Design System entry point
// Tokens
export * from './tokens';

// Primitives (added in subsequent tasks)
// export * from './primitives';

// Compositions (added in subsequent tasks)
// export * from './compositions';
```

**Step 5: Update root package.json**

Add `"design-system"` to workspaces array:

```json
{
  "workspaces": [
    "design-system",
    "web-ui",
    "e2e-tests"
  ]
}
```

**Step 6: Create Ladle config**

Create `design-system/.ladle/config.mjs`:

```javascript
export default {
  stories: 'src/**/*.stories.tsx',
  defaultStory: 'tokens--colors',
  addons: {
    theme: {
      enabled: true,
      defaultState: 'dark',
    },
  },
};
```

**Step 7: Install dependencies**

Run: `cd design-system && npm install`

**Step 8: Commit**

```bash
git add design-system package.json
git commit -m "feat(design-system): initialize package with Ladle"
```

---

### Task 1.2: Create color tokens

**Files:**
- Create: `design-system/src/tokens/colors.css`
- Create: `design-system/src/tokens/index.css`
- Create: `design-system/src/tokens/index.ts`
- Create: `design-system/src/tokens/Colors.stories.tsx`

**Step 1: Create colors.css**

```css
/* design-system/src/tokens/colors.css */

:root {
  /* === Base Palette (Dark Theme) === */
  --color-bg-base: #1a1816;
  --color-bg-surface: #242220;
  --color-bg-elevated: #2e2c29;
  --color-bg-overlay: rgba(26, 24, 22, 0.95);

  /* === Text === */
  --color-text-primary: #f0ebe3;
  --color-text-secondary: #a39e93;
  --color-text-dim: #6b665c;
  --color-text-inverse: #1a1816;

  /* === Semantic Colors === */
  --color-accent: #e6b450;
  --color-success: #7ec699;
  --color-warning: #e6b450;
  --color-error: #e05252;
  --color-info: #7eb8c9;

  /* === Plugin Accents === */
  --color-groove: #c9a227;

  /* === Borders === */
  --color-border: #3d3a36;
  --color-border-subtle: #2e2c29;
  --color-border-strong: #4d4a46;

  /* === Interactive States === */
  --color-hover: rgba(230, 180, 80, 0.1);
  --color-active: rgba(230, 180, 80, 0.2);
  --color-focus: rgba(230, 180, 80, 0.4);

  /* === Glow Effects === */
  --glow-amber: 0 0 20px rgba(230, 180, 80, 0.15);
  --glow-success: 0 0 20px rgba(126, 198, 153, 0.15);
  --glow-error: 0 0 20px rgba(224, 82, 82, 0.15);
}

/* === Light Theme === */
[data-theme="light"] {
  --color-bg-base: #f5f2ed;
  --color-bg-surface: #ffffff;
  --color-bg-elevated: #f9f7f4;
  --color-bg-overlay: rgba(245, 242, 237, 0.95);

  --color-text-primary: #1a1816;
  --color-text-secondary: #4a4640;
  --color-text-dim: #6b665c;
  --color-text-inverse: #f0ebe3;

  --color-border: #d4d0c8;
  --color-border-subtle: #e8e4dc;
  --color-border-strong: #b8b4ac;

  --color-hover: rgba(26, 24, 22, 0.05);
  --color-active: rgba(26, 24, 22, 0.1);
  --color-focus: rgba(230, 180, 80, 0.3);

  --glow-amber: 0 0 20px rgba(230, 180, 80, 0.1);
}
```

**Step 2: Create index.css**

```css
/* design-system/src/tokens/index.css */
@import './colors.css';
/* @import './typography.css'; - added in next task */
/* @import './spacing.css'; - added in next task */
```

**Step 3: Create index.ts**

```typescript
// design-system/src/tokens/index.ts
// Token exports for programmatic access

export const colors = {
  bgBase: 'var(--color-bg-base)',
  bgSurface: 'var(--color-bg-surface)',
  bgElevated: 'var(--color-bg-elevated)',
  textPrimary: 'var(--color-text-primary)',
  textSecondary: 'var(--color-text-secondary)',
  textDim: 'var(--color-text-dim)',
  accent: 'var(--color-accent)',
  success: 'var(--color-success)',
  warning: 'var(--color-warning)',
  error: 'var(--color-error)',
  info: 'var(--color-info)',
  groove: 'var(--color-groove)',
  border: 'var(--color-border)',
} as const;
```

**Step 4: Create Colors.stories.tsx**

```tsx
// design-system/src/tokens/Colors.stories.tsx
import '../index.css';

const colorGroups = {
  'Backgrounds': [
    { name: '--color-bg-base', label: 'Base' },
    { name: '--color-bg-surface', label: 'Surface' },
    { name: '--color-bg-elevated', label: 'Elevated' },
  ],
  'Text': [
    { name: '--color-text-primary', label: 'Primary' },
    { name: '--color-text-secondary', label: 'Secondary' },
    { name: '--color-text-dim', label: 'Dim' },
  ],
  'Semantic': [
    { name: '--color-accent', label: 'Accent (Amber)' },
    { name: '--color-success', label: 'Success' },
    { name: '--color-warning', label: 'Warning' },
    { name: '--color-error', label: 'Error' },
    { name: '--color-info', label: 'Info' },
  ],
  'Plugins': [
    { name: '--color-groove', label: 'groove Gold' },
  ],
};

function ColorSwatch({ name, label }: { name: string; label: string }) {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', marginBottom: '0.5rem' }}>
      <div
        style={{
          width: '3rem',
          height: '3rem',
          backgroundColor: `var(${name})`,
          borderRadius: '4px',
          border: '1px solid var(--color-border)',
        }}
      />
      <div>
        <div style={{ color: 'var(--color-text-primary)', fontWeight: 500 }}>{label}</div>
        <code style={{ color: 'var(--color-text-dim)', fontSize: '0.75rem' }}>{name}</code>
      </div>
    </div>
  );
}

export const Colors = () => (
  <div style={{ padding: '2rem', backgroundColor: 'var(--color-bg-base)', minHeight: '100vh' }}>
    {Object.entries(colorGroups).map(([group, colors]) => (
      <div key={group} style={{ marginBottom: '2rem' }}>
        <h2 style={{ color: 'var(--color-text-primary)', marginBottom: '1rem' }}>{group}</h2>
        {colors.map((color) => (
          <ColorSwatch key={color.name} {...color} />
        ))}
      </div>
    ))}
  </div>
);
```

**Step 5: Verify Ladle runs**

Run: `cd design-system && npm run dev`
Expected: Ladle opens at localhost:61000 showing color swatches

**Step 6: Commit**

```bash
git add design-system/src/tokens
git commit -m "feat(design-system): add color tokens with Ladle story"
```

---

### Task 1.3: Create typography tokens

**Files:**
- Create: `design-system/src/tokens/typography.css`
- Create: `design-system/src/tokens/Typography.stories.tsx`
- Modify: `design-system/src/tokens/index.css`

**Step 1: Create typography.css**

```css
/* design-system/src/tokens/typography.css */

:root {
  /* === Font Families === */
  --font-mono: 'JetBrains Mono', 'SF Mono', 'Cascadia Code', 'Menlo', monospace;
  --font-sans: -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;

  /* === Font Sizes === */
  --text-xs: 0.75rem;    /* 12px */
  --text-sm: 0.875rem;   /* 14px */
  --text-base: 1rem;     /* 16px */
  --text-lg: 1.125rem;   /* 18px */
  --text-xl: 1.25rem;    /* 20px */
  --text-2xl: 1.5rem;    /* 24px */

  /* === Line Heights === */
  --leading-tight: 1.25;
  --leading-normal: 1.5;
  --leading-relaxed: 1.75;

  /* === Letter Spacing === */
  --tracking-tight: -0.02em;
  --tracking-normal: 0;
  --tracking-wide: 0.02em;
}
```

**Step 2: Create Typography.stories.tsx**

```tsx
// design-system/src/tokens/Typography.stories.tsx
import '../index.css';

const samples = [
  { size: '--text-2xl', label: '2XL (24px)', sample: 'vibes groove' },
  { size: '--text-xl', label: 'XL (20px)', sample: 'Session Manager' },
  { size: '--text-lg', label: 'LG (18px)', sample: 'Event Stream' },
  { size: '--text-base', label: 'Base (16px)', sample: 'Body text for descriptions and content.' },
  { size: '--text-sm', label: 'SM (14px)', sample: 'Secondary text and metadata' },
  { size: '--text-xs', label: 'XS (12px)', sample: 'Timestamps and labels' },
];

export const Typography = () => (
  <div style={{ padding: '2rem', backgroundColor: 'var(--color-bg-base)', minHeight: '100vh' }}>
    <h2 style={{ color: 'var(--color-text-primary)', marginBottom: '2rem' }}>Typography Scale</h2>

    <div style={{ marginBottom: '3rem' }}>
      <h3 style={{ color: 'var(--color-text-secondary)', marginBottom: '1rem' }}>Monospace (Primary)</h3>
      {samples.map(({ size, label, sample }) => (
        <div key={size} style={{ marginBottom: '1.5rem' }}>
          <code style={{ color: 'var(--color-text-dim)', fontSize: 'var(--text-xs)' }}>{label}</code>
          <div style={{
            fontFamily: 'var(--font-mono)',
            fontSize: `var(${size})`,
            color: 'var(--color-text-primary)',
            marginTop: '0.25rem',
          }}>
            {sample}
          </div>
        </div>
      ))}
    </div>

    <div>
      <h3 style={{ color: 'var(--color-text-secondary)', marginBottom: '1rem' }}>Sans-serif (UI)</h3>
      {samples.slice(3).map(({ size, label, sample }) => (
        <div key={size} style={{ marginBottom: '1.5rem' }}>
          <code style={{ color: 'var(--color-text-dim)', fontSize: 'var(--text-xs)' }}>{label}</code>
          <div style={{
            fontFamily: 'var(--font-sans)',
            fontSize: `var(${size})`,
            color: 'var(--color-text-primary)',
            marginTop: '0.25rem',
          }}>
            {sample}
          </div>
        </div>
      ))}
    </div>
  </div>
);
```

**Step 3: Update index.css**

```css
/* design-system/src/tokens/index.css */
@import './colors.css';
@import './typography.css';
/* @import './spacing.css'; - added in next task */
```

**Step 4: Verify in Ladle**

Run: `cd design-system && npm run dev`
Expected: Typography story shows font samples

**Step 5: Commit**

```bash
git add design-system/src/tokens
git commit -m "feat(design-system): add typography tokens"
```

---

### Task 1.4: Create spacing tokens

**Files:**
- Create: `design-system/src/tokens/spacing.css`
- Create: `design-system/src/tokens/Spacing.stories.tsx`
- Modify: `design-system/src/tokens/index.css`

**Step 1: Create spacing.css**

```css
/* design-system/src/tokens/spacing.css */

:root {
  /* === Spacing Scale === */
  --space-0: 0;
  --space-1: 0.25rem;   /* 4px */
  --space-2: 0.5rem;    /* 8px */
  --space-3: 0.75rem;   /* 12px */
  --space-4: 1rem;      /* 16px */
  --space-5: 1.25rem;   /* 20px */
  --space-6: 1.5rem;    /* 24px */
  --space-8: 2rem;      /* 32px */
  --space-10: 2.5rem;   /* 40px */
  --space-12: 3rem;     /* 48px */
  --space-16: 4rem;     /* 64px */

  /* === Border Radius === */
  --radius-sm: 2px;
  --radius-md: 4px;
  --radius-lg: 8px;
  --radius-full: 9999px;

  /* === Transitions === */
  --transition-fast: 100ms ease;
  --transition-normal: 200ms ease;
  --transition-slow: 300ms ease;
}
```

**Step 2: Create Spacing.stories.tsx**

```tsx
// design-system/src/tokens/Spacing.stories.tsx
import '../index.css';

const spaces = [1, 2, 3, 4, 5, 6, 8, 10, 12, 16];

export const Spacing = () => (
  <div style={{ padding: '2rem', backgroundColor: 'var(--color-bg-base)', minHeight: '100vh' }}>
    <h2 style={{ color: 'var(--color-text-primary)', marginBottom: '2rem' }}>Spacing Scale</h2>

    {spaces.map((n) => (
      <div key={n} style={{ display: 'flex', alignItems: 'center', marginBottom: '0.5rem' }}>
        <code style={{ color: 'var(--color-text-dim)', width: '8rem', fontSize: 'var(--text-xs)' }}>
          --space-{n}
        </code>
        <div
          style={{
            width: `var(--space-${n})`,
            height: '1.5rem',
            backgroundColor: 'var(--color-accent)',
            borderRadius: 'var(--radius-sm)',
          }}
        />
        <span style={{ color: 'var(--color-text-secondary)', marginLeft: '1rem', fontSize: 'var(--text-sm)' }}>
          {n * 4}px
        </span>
      </div>
    ))}

    <h2 style={{ color: 'var(--color-text-primary)', margin: '2rem 0 1rem' }}>Border Radius</h2>
    <div style={{ display: 'flex', gap: '1rem' }}>
      {['sm', 'md', 'lg', 'full'].map((r) => (
        <div key={r} style={{ textAlign: 'center' }}>
          <div
            style={{
              width: '3rem',
              height: '3rem',
              backgroundColor: 'var(--color-accent)',
              borderRadius: `var(--radius-${r})`,
            }}
          />
          <code style={{ color: 'var(--color-text-dim)', fontSize: 'var(--text-xs)' }}>
            --radius-{r}
          </code>
        </div>
      ))}
    </div>
  </div>
);
```

**Step 3: Update index.css**

```css
/* design-system/src/tokens/index.css */
@import './colors.css';
@import './typography.css';
@import './spacing.css';
```

**Step 4: Commit**

```bash
git add design-system/src/tokens
git commit -m "feat(design-system): add spacing tokens"
```

---

## Phase 2: Primitive Components

### Task 2.1: Create Button primitive

**Files:**
- Create: `design-system/src/primitives/Button/Button.tsx`
- Create: `design-system/src/primitives/Button/Button.module.css`
- Create: `design-system/src/primitives/Button/Button.test.tsx`
- Create: `design-system/src/primitives/Button/Button.stories.tsx`
- Create: `design-system/src/primitives/Button/index.ts`
- Create: `design-system/src/primitives/index.ts`

**Step 1: Write the failing test**

```tsx
// design-system/src/primitives/Button/Button.test.tsx
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Button } from './Button';

describe('Button', () => {
  it('renders children', () => {
    render(<Button>Click me</Button>);
    expect(screen.getByRole('button')).toHaveTextContent('Click me');
  });

  it('applies primary variant by default', () => {
    render(<Button>Primary</Button>);
    expect(screen.getByRole('button')).toHaveClass('primary');
  });

  it('applies secondary variant', () => {
    render(<Button variant="secondary">Secondary</Button>);
    expect(screen.getByRole('button')).toHaveClass('secondary');
  });

  it('applies ghost variant', () => {
    render(<Button variant="ghost">Ghost</Button>);
    expect(screen.getByRole('button')).toHaveClass('ghost');
  });

  it('can be disabled', () => {
    render(<Button disabled>Disabled</Button>);
    expect(screen.getByRole('button')).toBeDisabled();
  });
});
```

**Step 2: Run test to verify it fails**

Run: `cd design-system && npm test -- Button.test.tsx`
Expected: FAIL - Cannot find module './Button'

**Step 3: Create Button.module.css**

```css
/* design-system/src/primitives/Button/Button.module.css */
.button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-4);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  font-weight: 500;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
  border: 1px solid transparent;
}

.button:focus-visible {
  outline: 2px solid var(--color-accent);
  outline-offset: 2px;
}

.button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Primary variant */
.primary {
  background-color: var(--color-accent);
  color: var(--color-text-inverse);
  border-color: var(--color-accent);
}

.primary:hover:not(:disabled) {
  filter: brightness(1.1);
  box-shadow: var(--glow-amber);
}

/* Secondary variant */
.secondary {
  background-color: transparent;
  color: var(--color-text-primary);
  border-color: var(--color-border);
}

.secondary:hover:not(:disabled) {
  border-color: var(--color-accent);
  color: var(--color-accent);
}

/* Ghost variant */
.ghost {
  background-color: transparent;
  color: var(--color-text-secondary);
  border-color: transparent;
}

.ghost:hover:not(:disabled) {
  background-color: var(--color-hover);
  color: var(--color-text-primary);
}

/* Sizes */
.sm {
  padding: var(--space-1) var(--space-3);
  font-size: var(--text-xs);
}

.lg {
  padding: var(--space-3) var(--space-6);
  font-size: var(--text-base);
}
```

**Step 4: Create Button.tsx**

```tsx
// design-system/src/primitives/Button/Button.tsx
import { ButtonHTMLAttributes, forwardRef } from 'react';
import styles from './Button.module.css';

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'ghost';
  size?: 'sm' | 'md' | 'lg';
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ variant = 'primary', size = 'md', className = '', children, ...props }, ref) => {
    const classes = [
      styles.button,
      styles[variant],
      size !== 'md' && styles[size],
      className,
    ].filter(Boolean).join(' ');

    return (
      <button ref={ref} className={classes} {...props}>
        {children}
      </button>
    );
  }
);

Button.displayName = 'Button';
```

**Step 5: Create index.ts**

```typescript
// design-system/src/primitives/Button/index.ts
export { Button } from './Button';
export type { ButtonProps } from './Button';
```

**Step 6: Create primitives index**

```typescript
// design-system/src/primitives/index.ts
export * from './Button';
```

**Step 7: Run test to verify it passes**

Run: `cd design-system && npm test -- Button.test.tsx`
Expected: PASS

**Step 8: Create Button.stories.tsx**

```tsx
// design-system/src/primitives/Button/Button.stories.tsx
import '../../tokens/index.css';
import { Button } from './Button';

export const Variants = () => (
  <div style={{ display: 'flex', gap: '1rem', padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <Button variant="primary">Primary</Button>
    <Button variant="secondary">Secondary</Button>
    <Button variant="ghost">Ghost</Button>
  </div>
);

export const Sizes = () => (
  <div style={{ display: 'flex', gap: '1rem', alignItems: 'center', padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <Button size="sm">Small</Button>
    <Button size="md">Medium</Button>
    <Button size="lg">Large</Button>
  </div>
);

export const States = () => (
  <div style={{ display: 'flex', gap: '1rem', padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <Button>Default</Button>
    <Button disabled>Disabled</Button>
  </div>
);
```

**Step 9: Verify in Ladle**

Run: `cd design-system && npm run dev`
Expected: Button stories appear with all variants

**Step 10: Commit**

```bash
git add design-system/src/primitives
git commit -m "feat(design-system): add Button primitive"
```

---

### Task 2.2: Create Badge primitive

**Files:**
- Create: `design-system/src/primitives/Badge/Badge.tsx`
- Create: `design-system/src/primitives/Badge/Badge.module.css`
- Create: `design-system/src/primitives/Badge/Badge.test.tsx`
- Create: `design-system/src/primitives/Badge/Badge.stories.tsx`
- Create: `design-system/src/primitives/Badge/index.ts`
- Modify: `design-system/src/primitives/index.ts`

**Step 1: Write the failing test**

```tsx
// design-system/src/primitives/Badge/Badge.test.tsx
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Badge } from './Badge';

describe('Badge', () => {
  it('renders children', () => {
    render(<Badge>Connected</Badge>);
    expect(screen.getByText('Connected')).toBeInTheDocument();
  });

  it('applies idle status by default', () => {
    render(<Badge>Idle</Badge>);
    expect(screen.getByText('Idle')).toHaveClass('idle');
  });

  it('applies success status', () => {
    render(<Badge status="success">Connected</Badge>);
    expect(screen.getByText('Connected')).toHaveClass('success');
  });

  it('applies warning status', () => {
    render(<Badge status="warning">Processing</Badge>);
    expect(screen.getByText('Processing')).toHaveClass('warning');
  });

  it('applies error status', () => {
    render(<Badge status="error">Failed</Badge>);
    expect(screen.getByText('Failed')).toHaveClass('error');
  });
});
```

**Step 2: Run test to verify it fails**

Run: `cd design-system && npm test -- Badge.test.tsx`
Expected: FAIL

**Step 3: Create Badge.module.css**

```css
/* design-system/src/primitives/Badge/Badge.module.css */
.badge {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  padding: var(--space-1) var(--space-2);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  font-weight: 500;
  border-radius: var(--radius-full);
  white-space: nowrap;
}

.idle {
  background-color: var(--color-border);
  color: var(--color-text-dim);
}

.success {
  background-color: rgba(126, 198, 153, 0.2);
  color: var(--color-success);
}

.warning {
  background-color: rgba(230, 180, 80, 0.2);
  color: var(--color-warning);
}

.error {
  background-color: rgba(224, 82, 82, 0.2);
  color: var(--color-error);
}

.info {
  background-color: rgba(126, 184, 201, 0.2);
  color: var(--color-info);
}

.accent {
  background-color: rgba(230, 180, 80, 0.2);
  color: var(--color-accent);
}
```

**Step 4: Create Badge.tsx**

```tsx
// design-system/src/primitives/Badge/Badge.tsx
import { HTMLAttributes, forwardRef } from 'react';
import styles from './Badge.module.css';

export interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
  status?: 'idle' | 'success' | 'warning' | 'error' | 'info' | 'accent';
}

export const Badge = forwardRef<HTMLSpanElement, BadgeProps>(
  ({ status = 'idle', className = '', children, ...props }, ref) => {
    const classes = [styles.badge, styles[status], className].filter(Boolean).join(' ');

    return (
      <span ref={ref} className={classes} {...props}>
        {children}
      </span>
    );
  }
);

Badge.displayName = 'Badge';
```

**Step 5: Create index.ts**

```typescript
// design-system/src/primitives/Badge/index.ts
export { Badge } from './Badge';
export type { BadgeProps } from './Badge';
```

**Step 6: Update primitives index**

```typescript
// design-system/src/primitives/index.ts
export * from './Button';
export * from './Badge';
```

**Step 7: Run test to verify it passes**

Run: `cd design-system && npm test -- Badge.test.tsx`
Expected: PASS

**Step 8: Create Badge.stories.tsx**

```tsx
// design-system/src/primitives/Badge/Badge.stories.tsx
import '../../tokens/index.css';
import { Badge } from './Badge';

export const Statuses = () => (
  <div style={{ display: 'flex', gap: '1rem', padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <Badge status="idle">Idle</Badge>
    <Badge status="success">Connected</Badge>
    <Badge status="warning">Processing</Badge>
    <Badge status="error">Failed</Badge>
    <Badge status="info">Info</Badge>
    <Badge status="accent">Accent</Badge>
  </div>
);
```

**Step 9: Commit**

```bash
git add design-system/src/primitives/Badge design-system/src/primitives/index.ts
git commit -m "feat(design-system): add Badge primitive"
```

---

### Task 2.3: Create Panel primitive

**Files:**
- Create: `design-system/src/primitives/Panel/Panel.tsx`
- Create: `design-system/src/primitives/Panel/Panel.module.css`
- Create: `design-system/src/primitives/Panel/Panel.test.tsx`
- Create: `design-system/src/primitives/Panel/Panel.stories.tsx`
- Create: `design-system/src/primitives/Panel/index.ts`
- Modify: `design-system/src/primitives/index.ts`

**Step 1: Write the failing test**

```tsx
// design-system/src/primitives/Panel/Panel.test.tsx
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Panel } from './Panel';

describe('Panel', () => {
  it('renders children', () => {
    render(<Panel>Content</Panel>);
    expect(screen.getByText('Content')).toBeInTheDocument();
  });

  it('renders title when provided', () => {
    render(<Panel title="Settings">Content</Panel>);
    expect(screen.getByText('Settings')).toBeInTheDocument();
  });

  it('applies default variant', () => {
    const { container } = render(<Panel>Content</Panel>);
    expect(container.firstChild).toHaveClass('default');
  });

  it('applies elevated variant', () => {
    const { container } = render(<Panel variant="elevated">Content</Panel>);
    expect(container.firstChild).toHaveClass('elevated');
  });

  it('applies inset variant', () => {
    const { container } = render(<Panel variant="inset">Content</Panel>);
    expect(container.firstChild).toHaveClass('inset');
  });
});
```

**Step 2: Run test to verify it fails**

Run: `cd design-system && npm test -- Panel.test.tsx`
Expected: FAIL

**Step 3: Create Panel.module.css**

```css
/* design-system/src/primitives/Panel/Panel.module.css */
.panel {
  border-radius: var(--radius-lg);
  overflow: hidden;
}

.default {
  background-color: var(--color-bg-surface);
  border: 1px solid var(--color-border);
}

.elevated {
  background-color: var(--color-bg-elevated);
  border: 1px solid var(--color-border-strong);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.inset {
  background-color: var(--color-bg-base);
  border: 1px solid var(--color-border-subtle);
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-3) var(--space-4);
  border-bottom: 1px solid var(--color-border-subtle);
}

.title {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  font-weight: 500;
  color: var(--color-text-primary);
  margin: 0;
}

.content {
  padding: var(--space-4);
}

.noPadding .content {
  padding: 0;
}
```

**Step 4: Create Panel.tsx**

```tsx
// design-system/src/primitives/Panel/Panel.tsx
import { HTMLAttributes, ReactNode, forwardRef } from 'react';
import styles from './Panel.module.css';

export interface PanelProps extends HTMLAttributes<HTMLDivElement> {
  title?: string;
  variant?: 'default' | 'elevated' | 'inset';
  actions?: ReactNode;
  noPadding?: boolean;
}

export const Panel = forwardRef<HTMLDivElement, PanelProps>(
  ({ title, variant = 'default', actions, noPadding, className = '', children, ...props }, ref) => {
    const classes = [
      styles.panel,
      styles[variant],
      noPadding && styles.noPadding,
      className,
    ].filter(Boolean).join(' ');

    return (
      <div ref={ref} className={classes} {...props}>
        {title && (
          <div className={styles.header}>
            <h3 className={styles.title}>{title}</h3>
            {actions}
          </div>
        )}
        <div className={styles.content}>{children}</div>
      </div>
    );
  }
);

Panel.displayName = 'Panel';
```

**Step 5: Create index.ts and update exports**

```typescript
// design-system/src/primitives/Panel/index.ts
export { Panel } from './Panel';
export type { PanelProps } from './Panel';
```

```typescript
// design-system/src/primitives/index.ts
export * from './Button';
export * from './Badge';
export * from './Panel';
```

**Step 6: Run test to verify it passes**

Run: `cd design-system && npm test -- Panel.test.tsx`
Expected: PASS

**Step 7: Create Panel.stories.tsx**

```tsx
// design-system/src/primitives/Panel/Panel.stories.tsx
import '../../tokens/index.css';
import { Panel } from './Panel';
import { Button } from '../Button';

export const Variants = () => (
  <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem', padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <Panel title="Default Panel" variant="default">
      <p style={{ color: 'var(--color-text-secondary)', margin: 0 }}>This is the default panel variant.</p>
    </Panel>
    <Panel title="Elevated Panel" variant="elevated">
      <p style={{ color: 'var(--color-text-secondary)', margin: 0 }}>This panel has elevation and stronger borders.</p>
    </Panel>
    <Panel title="Inset Panel" variant="inset">
      <p style={{ color: 'var(--color-text-secondary)', margin: 0 }}>This panel is recessed into the background.</p>
    </Panel>
  </div>
);

export const WithActions = () => (
  <div style={{ padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <Panel
      title="Session Events"
      actions={<Button size="sm" variant="secondary">Refresh</Button>}
    >
      <p style={{ color: 'var(--color-text-secondary)', margin: 0 }}>Panel with action buttons in header.</p>
    </Panel>
  </div>
);
```

**Step 8: Commit**

```bash
git add design-system/src/primitives/Panel design-system/src/primitives/index.ts
git commit -m "feat(design-system): add Panel primitive"
```

---

### Task 2.4: Create Text primitive

**Files:**
- Create: `design-system/src/primitives/Text/Text.tsx`
- Create: `design-system/src/primitives/Text/Text.module.css`
- Create: `design-system/src/primitives/Text/Text.test.tsx`
- Create: `design-system/src/primitives/Text/Text.stories.tsx`
- Create: `design-system/src/primitives/Text/index.ts`
- Modify: `design-system/src/primitives/index.ts`

**Step 1: Write the failing test**

```tsx
// design-system/src/primitives/Text/Text.test.tsx
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Text } from './Text';

describe('Text', () => {
  it('renders children', () => {
    render(<Text>Hello world</Text>);
    expect(screen.getByText('Hello world')).toBeInTheDocument();
  });

  it('applies normal intensity by default', () => {
    render(<Text>Normal</Text>);
    expect(screen.getByText('Normal')).toHaveClass('normal');
  });

  it('applies high intensity', () => {
    render(<Text intensity="high">Important</Text>);
    expect(screen.getByText('Important')).toHaveClass('high');
  });

  it('applies dim intensity', () => {
    render(<Text intensity="dim">Metadata</Text>);
    expect(screen.getByText('Metadata')).toHaveClass('dim');
  });

  it('applies mono font', () => {
    render(<Text mono>code</Text>);
    expect(screen.getByText('code')).toHaveClass('mono');
  });

  it('renders as different elements', () => {
    render(<Text as="h1">Heading</Text>);
    expect(screen.getByRole('heading', { level: 1 })).toBeInTheDocument();
  });
});
```

**Step 2: Run test to verify it fails**

Run: `cd design-system && npm test -- Text.test.tsx`
Expected: FAIL

**Step 3: Create Text.module.css**

```css
/* design-system/src/primitives/Text/Text.module.css */
.text {
  margin: 0;
  font-family: var(--font-sans);
  line-height: var(--leading-normal);
}

/* Intensity */
.high {
  color: var(--color-text-primary);
  font-weight: 500;
}

.normal {
  color: var(--color-text-secondary);
}

.dim {
  color: var(--color-text-dim);
}

/* Monospace */
.mono {
  font-family: var(--font-mono);
}

/* Sizes */
.xs { font-size: var(--text-xs); }
.sm { font-size: var(--text-sm); }
.base { font-size: var(--text-base); }
.lg { font-size: var(--text-lg); }
.xl { font-size: var(--text-xl); }
.xxl { font-size: var(--text-2xl); }
```

**Step 4: Create Text.tsx**

```tsx
// design-system/src/primitives/Text/Text.tsx
import { ElementType, HTMLAttributes, forwardRef } from 'react';
import styles from './Text.module.css';

export interface TextProps extends HTMLAttributes<HTMLElement> {
  as?: ElementType;
  intensity?: 'high' | 'normal' | 'dim';
  size?: 'xs' | 'sm' | 'base' | 'lg' | 'xl' | '2xl';
  mono?: boolean;
}

export const Text = forwardRef<HTMLElement, TextProps>(
  ({ as: Component = 'span', intensity = 'normal', size, mono, className = '', children, ...props }, ref) => {
    const classes = [
      styles.text,
      styles[intensity],
      size && styles[size === '2xl' ? 'xxl' : size],
      mono && styles.mono,
      className,
    ].filter(Boolean).join(' ');

    return (
      <Component ref={ref} className={classes} {...props}>
        {children}
      </Component>
    );
  }
);

Text.displayName = 'Text';
```

**Step 5: Create index.ts and update exports**

```typescript
// design-system/src/primitives/Text/index.ts
export { Text } from './Text';
export type { TextProps } from './Text';
```

```typescript
// design-system/src/primitives/index.ts
export * from './Button';
export * from './Badge';
export * from './Panel';
export * from './Text';
```

**Step 6: Run test to verify it passes**

Run: `cd design-system && npm test -- Text.test.tsx`
Expected: PASS

**Step 7: Create Text.stories.tsx**

```tsx
// design-system/src/primitives/Text/Text.stories.tsx
import '../../tokens/index.css';
import { Text } from './Text';

export const Intensities = () => (
  <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <Text intensity="high">High intensity - Primary content</Text>
    <Text intensity="normal">Normal intensity - Secondary content</Text>
    <Text intensity="dim">Dim intensity - Metadata and timestamps</Text>
  </div>
);

export const Monospace = () => (
  <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <Text>Sans-serif text (default)</Text>
    <Text mono>Monospace text (code, IDs, values)</Text>
    <Text mono intensity="dim">session_id: abc123</Text>
  </div>
);

export const Sizes = () => (
  <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <Text size="2xl" intensity="high">2XL Heading</Text>
    <Text size="xl" intensity="high">XL Heading</Text>
    <Text size="lg" intensity="high">LG Heading</Text>
    <Text size="base">Base text</Text>
    <Text size="sm">Small text</Text>
    <Text size="xs" intensity="dim">Extra small text</Text>
  </div>
);
```

**Step 8: Commit**

```bash
git add design-system/src/primitives/Text design-system/src/primitives/index.ts
git commit -m "feat(design-system): add Text primitive"
```

---

### Task 2.5: Create StatusIndicator primitive

**Files:**
- Create: `design-system/src/primitives/StatusIndicator/StatusIndicator.tsx`
- Create: `design-system/src/primitives/StatusIndicator/StatusIndicator.module.css`
- Create: `design-system/src/primitives/StatusIndicator/StatusIndicator.test.tsx`
- Create: `design-system/src/primitives/StatusIndicator/StatusIndicator.stories.tsx`
- Create: `design-system/src/primitives/StatusIndicator/index.ts`
- Modify: `design-system/src/primitives/index.ts`

**Step 1: Write the failing test**

```tsx
// design-system/src/primitives/StatusIndicator/StatusIndicator.test.tsx
import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import { StatusIndicator } from './StatusIndicator';

describe('StatusIndicator', () => {
  it('renders a dot', () => {
    const { container } = render(<StatusIndicator state="live" />);
    expect(container.querySelector('.dot')).toBeInTheDocument();
  });

  it('applies live state', () => {
    const { container } = render(<StatusIndicator state="live" />);
    expect(container.firstChild).toHaveClass('live');
  });

  it('applies paused state', () => {
    const { container } = render(<StatusIndicator state="paused" />);
    expect(container.firstChild).toHaveClass('paused');
  });

  it('applies offline state', () => {
    const { container } = render(<StatusIndicator state="offline" />);
    expect(container.firstChild).toHaveClass('offline');
  });

  it('renders label when provided', () => {
    const { container } = render(<StatusIndicator state="live" label="Connected" />);
    expect(container.textContent).toContain('Connected');
  });
});
```

**Step 2: Run test to verify it fails**

Run: `cd design-system && npm test -- StatusIndicator.test.tsx`
Expected: FAIL

**Step 3: Create StatusIndicator.module.css**

```css
/* design-system/src/primitives/StatusIndicator/StatusIndicator.module.css */
.indicator {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: var(--radius-full);
}

.label {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  color: var(--color-text-secondary);
}

/* States */
.live .dot {
  background-color: var(--color-success);
  box-shadow: var(--glow-success);
  animation: pulse 2s ease-in-out infinite;
}

.paused .dot {
  background-color: var(--color-warning);
}

.offline .dot {
  background-color: var(--color-text-dim);
}

.error .dot {
  background-color: var(--color-error);
  box-shadow: var(--glow-error);
  animation: pulse 1s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}
```

**Step 4: Create StatusIndicator.tsx**

```tsx
// design-system/src/primitives/StatusIndicator/StatusIndicator.tsx
import { HTMLAttributes, forwardRef } from 'react';
import styles from './StatusIndicator.module.css';

export interface StatusIndicatorProps extends HTMLAttributes<HTMLDivElement> {
  state: 'live' | 'paused' | 'offline' | 'error';
  label?: string;
}

export const StatusIndicator = forwardRef<HTMLDivElement, StatusIndicatorProps>(
  ({ state, label, className = '', ...props }, ref) => {
    const classes = [styles.indicator, styles[state], className].filter(Boolean).join(' ');

    return (
      <div ref={ref} className={classes} {...props}>
        <span className={styles.dot} />
        {label && <span className={styles.label}>{label}</span>}
      </div>
    );
  }
);

StatusIndicator.displayName = 'StatusIndicator';
```

**Step 5: Create index.ts and update exports**

```typescript
// design-system/src/primitives/StatusIndicator/index.ts
export { StatusIndicator } from './StatusIndicator';
export type { StatusIndicatorProps } from './StatusIndicator';
```

```typescript
// design-system/src/primitives/index.ts
export * from './Button';
export * from './Badge';
export * from './Panel';
export * from './Text';
export * from './StatusIndicator';
```

**Step 6: Run test to verify it passes**

Run: `cd design-system && npm test -- StatusIndicator.test.tsx`
Expected: PASS

**Step 7: Create StatusIndicator.stories.tsx**

```tsx
// design-system/src/primitives/StatusIndicator/StatusIndicator.stories.tsx
import '../../tokens/index.css';
import { StatusIndicator } from './StatusIndicator';

export const States = () => (
  <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem', padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <StatusIndicator state="live" label="Live" />
    <StatusIndicator state="paused" label="Paused" />
    <StatusIndicator state="offline" label="Offline" />
    <StatusIndicator state="error" label="Error" />
  </div>
);

export const DotsOnly = () => (
  <div style={{ display: 'flex', gap: '1rem', padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <StatusIndicator state="live" />
    <StatusIndicator state="paused" />
    <StatusIndicator state="offline" />
    <StatusIndicator state="error" />
  </div>
);
```

**Step 8: Commit**

```bash
git add design-system/src/primitives/StatusIndicator design-system/src/primitives/index.ts
git commit -m "feat(design-system): add StatusIndicator primitive"
```

---

### Task 2.6: Update design-system exports

**Files:**
- Modify: `design-system/src/index.ts`

**Step 1: Update main exports**

```typescript
// design-system/src/index.ts
// Design System entry point

// Tokens
export * from './tokens';

// Primitives
export * from './primitives';

// Compositions (added in Phase 2)
// export * from './compositions';
```

**Step 2: Run all tests**

Run: `cd design-system && npm test`
Expected: All tests PASS

**Step 3: Commit**

```bash
git add design-system/src/index.ts
git commit -m "feat(design-system): export all primitives from main entry"
```

---

## Phase 3: Composition Components

### Task 3.1: Create SessionCard composition

**Files:**
- Create: `design-system/src/compositions/SessionCard/SessionCard.tsx`
- Create: `design-system/src/compositions/SessionCard/SessionCard.module.css`
- Create: `design-system/src/compositions/SessionCard/SessionCard.test.tsx`
- Create: `design-system/src/compositions/SessionCard/SessionCard.stories.tsx`
- Create: `design-system/src/compositions/SessionCard/index.ts`
- Create: `design-system/src/compositions/index.ts`

**Step 1: Write the failing test**

```tsx
// design-system/src/compositions/SessionCard/SessionCard.test.tsx
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { SessionCard } from './SessionCard';

describe('SessionCard', () => {
  const defaultProps = {
    id: 'sess-abc123',
    status: 'processing' as const,
    updatedAt: new Date('2024-01-01T12:00:00Z'),
  };

  it('renders session id', () => {
    render(<SessionCard {...defaultProps} />);
    expect(screen.getByText('sess-abc123')).toBeInTheDocument();
  });

  it('renders session name when provided', () => {
    render(<SessionCard {...defaultProps} name="auth-refactor" />);
    expect(screen.getByText('auth-refactor')).toBeInTheDocument();
  });

  it('renders status badge', () => {
    render(<SessionCard {...defaultProps} status="processing" />);
    expect(screen.getByText('processing')).toBeInTheDocument();
  });

  it('renders subscriber count', () => {
    render(<SessionCard {...defaultProps} subscribers={3} />);
    expect(screen.getByText('3')).toBeInTheDocument();
  });

  it('calls onClick when clicked', () => {
    const onClick = vi.fn();
    render(<SessionCard {...defaultProps} onClick={onClick} />);
    fireEvent.click(screen.getByRole('article'));
    expect(onClick).toHaveBeenCalled();
  });
});
```

**Step 2: Run test to verify it fails**

Run: `cd design-system && npm test -- SessionCard.test.tsx`
Expected: FAIL

**Step 3: Create SessionCard.module.css**

```css
/* design-system/src/compositions/SessionCard/SessionCard.module.css */
.card {
  display: block;
  background-color: var(--color-bg-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--space-4);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.card:hover {
  border-color: var(--color-accent);
  transform: translateY(-2px);
  box-shadow: var(--glow-amber);
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: var(--space-3);
}

.title {
  font-family: var(--font-mono);
  font-size: var(--text-base);
  font-weight: 500;
  color: var(--color-text-primary);
  margin: 0;
}

.id {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  color: var(--color-text-dim);
  margin-top: var(--space-1);
}

.meta {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  font-size: var(--text-sm);
  color: var(--color-text-secondary);
}

.subscribers {
  display: flex;
  align-items: center;
  gap: var(--space-1);
}

.subscriberIcon {
  opacity: 0.7;
}

.time {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  color: var(--color-text-dim);
}
```

**Step 4: Create SessionCard.tsx**

```tsx
// design-system/src/compositions/SessionCard/SessionCard.tsx
import { forwardRef, HTMLAttributes } from 'react';
import { Badge } from '../../primitives/Badge';
import styles from './SessionCard.module.css';

export interface SessionCardProps extends HTMLAttributes<HTMLElement> {
  id: string;
  name?: string;
  status: 'idle' | 'processing' | 'waiting' | 'finished' | 'failed';
  subscribers?: number;
  updatedAt: Date;
}

const statusMap = {
  idle: 'idle',
  processing: 'accent',
  waiting: 'warning',
  finished: 'success',
  failed: 'error',
} as const;

export const SessionCard = forwardRef<HTMLElement, SessionCardProps>(
  ({ id, name, status, subscribers = 0, updatedAt, className = '', onClick, ...props }, ref) => {
    const classes = [styles.card, className].filter(Boolean).join(' ');

    const timeAgo = formatTimeAgo(updatedAt);

    return (
      <article ref={ref} className={classes} onClick={onClick} {...props}>
        <div className={styles.header}>
          <div>
            {name && <h3 className={styles.title}>{name}</h3>}
            <div className={styles.id}>{id}</div>
          </div>
          <Badge status={statusMap[status]}>{status}</Badge>
        </div>
        <div className={styles.meta}>
          <div className={styles.subscribers}>
            <span className={styles.subscriberIcon}>👤</span>
            <span>{subscribers}</span>
          </div>
          <span className={styles.time}>{timeAgo}</span>
        </div>
      </article>
    );
  }
);

SessionCard.displayName = 'SessionCard';

function formatTimeAgo(date: Date): string {
  const seconds = Math.floor((Date.now() - date.getTime()) / 1000);
  if (seconds < 60) return 'just now';
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}
```

**Step 5: Create index files**

```typescript
// design-system/src/compositions/SessionCard/index.ts
export { SessionCard } from './SessionCard';
export type { SessionCardProps } from './SessionCard';
```

```typescript
// design-system/src/compositions/index.ts
export * from './SessionCard';
```

**Step 6: Run test to verify it passes**

Run: `cd design-system && npm test -- SessionCard.test.tsx`
Expected: PASS

**Step 7: Create SessionCard.stories.tsx**

```tsx
// design-system/src/compositions/SessionCard/SessionCard.stories.tsx
import '../../tokens/index.css';
import { SessionCard } from './SessionCard';

export const Default = () => (
  <div style={{ padding: '2rem', backgroundColor: 'var(--color-bg-base)', maxWidth: '400px' }}>
    <SessionCard
      id="sess-abc123"
      name="auth-refactor"
      status="processing"
      subscribers={2}
      updatedAt={new Date(Date.now() - 1000 * 60 * 5)}
    />
  </div>
);

export const AllStatuses = () => (
  <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem', padding: '2rem', backgroundColor: 'var(--color-bg-base)', maxWidth: '400px' }}>
    <SessionCard id="sess-1" status="idle" subscribers={0} updatedAt={new Date()} />
    <SessionCard id="sess-2" name="feature-work" status="processing" subscribers={1} updatedAt={new Date()} />
    <SessionCard id="sess-3" name="bug-fix" status="waiting" subscribers={2} updatedAt={new Date()} />
    <SessionCard id="sess-4" status="finished" subscribers={1} updatedAt={new Date(Date.now() - 1000 * 60 * 60)} />
    <SessionCard id="sess-5" name="failed-task" status="failed" subscribers={0} updatedAt={new Date(Date.now() - 1000 * 60 * 60 * 24)} />
  </div>
);
```

**Step 8: Commit**

```bash
git add design-system/src/compositions
git commit -m "feat(design-system): add SessionCard composition"
```

---

### Task 3.2: Create Header composition

**Files:**
- Create: `design-system/src/compositions/Header/Header.tsx`
- Create: `design-system/src/compositions/Header/Header.module.css`
- Create: `design-system/src/compositions/Header/Header.test.tsx`
- Create: `design-system/src/compositions/Header/Header.stories.tsx`
- Create: `design-system/src/compositions/Header/index.ts`
- Modify: `design-system/src/compositions/index.ts`

**Step 1: Write the failing test**

```tsx
// design-system/src/compositions/Header/Header.test.tsx
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Header } from './Header';

describe('Header', () => {
  it('renders logo', () => {
    render(<Header />);
    expect(screen.getByText('vibes')).toBeInTheDocument();
  });

  it('renders nav items', () => {
    render(<Header navItems={[{ label: 'Sessions', href: '/sessions' }]} />);
    expect(screen.getByText('Sessions')).toBeInTheDocument();
  });

  it('renders identity when provided', () => {
    render(<Header identity={{ email: 'user@example.com' }} />);
    expect(screen.getByText('user@example.com')).toBeInTheDocument();
  });

  it('renders local badge when isLocal', () => {
    render(<Header isLocal />);
    expect(screen.getByText('Local')).toBeInTheDocument();
  });

  it('calls onThemeToggle when theme button clicked', () => {
    const onToggle = vi.fn();
    render(<Header theme="dark" onThemeToggle={onToggle} />);
    fireEvent.click(screen.getByLabelText('Toggle theme'));
    expect(onToggle).toHaveBeenCalled();
  });
});
```

**Step 2: Run test to verify it fails**

Run: `cd design-system && npm test -- Header.test.tsx`
Expected: FAIL

**Step 3: Create Header.module.css**

```css
/* design-system/src/compositions/Header/Header.module.css */
.header {
  display: flex;
  align-items: center;
  padding: var(--space-3) var(--space-6);
  background-color: var(--color-bg-surface);
  border-bottom: 1px solid var(--color-border);
}

.logo {
  font-family: var(--font-mono);
  font-size: var(--text-lg);
  font-weight: 600;
  color: var(--color-accent);
  text-decoration: none;
  margin-right: var(--space-8);
}

.logo:hover {
  text-shadow: var(--glow-amber);
}

.nav {
  display: flex;
  align-items: center;
  gap: var(--space-6);
  flex: 1;
}

.navLink {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--color-text-secondary);
  text-decoration: none;
  transition: color var(--transition-fast);
}

.navLink:hover {
  color: var(--color-text-primary);
}

.navLinkActive {
  color: var(--color-accent);
}

.grooveLink {
  color: var(--color-groove);
}

.grooveLink:hover {
  color: var(--color-groove);
  text-shadow: 0 0 8px rgba(201, 162, 39, 0.4);
}

.actions {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  margin-left: auto;
}

.identity {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--color-text-secondary);
}

.themeToggle {
  background: none;
  border: none;
  color: var(--color-text-secondary);
  cursor: pointer;
  padding: var(--space-2);
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}

.themeToggle:hover {
  background-color: var(--color-hover);
  color: var(--color-text-primary);
}
```

**Step 4: Create Header.tsx**

```tsx
// design-system/src/compositions/Header/Header.tsx
import { forwardRef, HTMLAttributes } from 'react';
import { Badge } from '../../primitives/Badge';
import styles from './Header.module.css';

export interface NavItem {
  label: string;
  href: string;
  isActive?: boolean;
  isGroove?: boolean;
}

export interface HeaderProps extends HTMLAttributes<HTMLElement> {
  navItems?: NavItem[];
  identity?: { email: string; provider?: string };
  isLocal?: boolean;
  theme?: 'dark' | 'light';
  onThemeToggle?: () => void;
  renderLink?: (props: { href: string; className: string; children: React.ReactNode }) => React.ReactNode;
}

export const Header = forwardRef<HTMLElement, HeaderProps>(
  ({ navItems = [], identity, isLocal, theme = 'dark', onThemeToggle, renderLink, className = '', ...props }, ref) => {
    const classes = [styles.header, className].filter(Boolean).join(' ');

    const Link = renderLink || (({ href, className, children }) => (
      <a href={href} className={className}>{children}</a>
    ));

    return (
      <header ref={ref} className={classes} {...props}>
        <Link href="/" className={styles.logo}>◈ vibes</Link>

        <nav className={styles.nav}>
          {navItems.map((item) => (
            <Link
              key={item.href}
              href={item.href}
              className={[
                styles.navLink,
                item.isActive && styles.navLinkActive,
                item.isGroove && styles.grooveLink,
              ].filter(Boolean).join(' ')}
            >
              {item.isGroove ? '◉ ' : ''}{item.label}
            </Link>
          ))}
        </nav>

        <div className={styles.actions}>
          {isLocal && <Badge status="idle">Local</Badge>}
          {identity && <span className={styles.identity}>{identity.email}</span>}
          {onThemeToggle && (
            <button
              className={styles.themeToggle}
              onClick={onThemeToggle}
              aria-label="Toggle theme"
            >
              {theme === 'dark' ? '☀' : '🌙'}
            </button>
          )}
        </div>
      </header>
    );
  }
);

Header.displayName = 'Header';
```

**Step 5: Create index and update exports**

```typescript
// design-system/src/compositions/Header/index.ts
export { Header } from './Header';
export type { HeaderProps, NavItem } from './Header';
```

```typescript
// design-system/src/compositions/index.ts
export * from './SessionCard';
export * from './Header';
```

**Step 6: Run test to verify it passes**

Run: `cd design-system && npm test -- Header.test.tsx`
Expected: PASS

**Step 7: Create Header.stories.tsx**

```tsx
// design-system/src/compositions/Header/Header.stories.tsx
import { useState } from 'react';
import '../../tokens/index.css';
import { Header } from './Header';

const navItems = [
  { label: 'Sessions', href: '/sessions', isActive: true },
  { label: 'Streams', href: '/streams' },
  { label: 'groove', href: '/groove', isGroove: true },
];

export const Default = () => (
  <Header navItems={navItems} isLocal />
);

export const WithIdentity = () => (
  <Header
    navItems={navItems}
    identity={{ email: 'user@example.com', provider: 'google' }}
  />
);

export const WithThemeToggle = () => {
  const [theme, setTheme] = useState<'dark' | 'light'>('dark');
  return (
    <div data-theme={theme} style={{ backgroundColor: 'var(--color-bg-base)' }}>
      <Header
        navItems={navItems}
        theme={theme}
        onThemeToggle={() => setTheme(t => t === 'dark' ? 'light' : 'dark')}
        isLocal
      />
    </div>
  );
};
```

**Step 8: Commit**

```bash
git add design-system/src/compositions/Header design-system/src/compositions/index.ts
git commit -m "feat(design-system): add Header composition"
```

---

## Phase 4: Server Integration

### Task 4.1: Add firehose WebSocket endpoint

**Files:**
- Create: `vibes-server/src/handlers/firehose.rs`
- Modify: `vibes-server/src/handlers/mod.rs`
- Modify: `vibes-server/src/lib.rs`

**Step 1: Create firehose handler**

```rust
// vibes-server/src/handlers/firehose.rs
//! WebSocket handler for firehose event streaming

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::broadcast;
use vibes_core::VibesEvent;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct FirehoseQuery {
    /// Filter by event types (comma-separated)
    #[serde(default)]
    pub types: Option<String>,
    /// Filter by session ID
    #[serde(default)]
    pub session: Option<String>,
}

/// WebSocket upgrade handler for firehose
pub async fn firehose_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<FirehoseQuery>,
) -> Response {
    ws.on_upgrade(move |socket| handle_firehose(socket, state, query))
}

async fn handle_firehose(socket: WebSocket, state: AppState, query: FirehoseQuery) {
    let (mut sender, mut receiver) = socket.split();
    let mut events_rx = state.subscribe_events();

    // Parse filter types
    let filter_types: Option<Vec<String>> = query.types.map(|t| {
        t.split(',').map(|s| s.trim().to_string()).collect()
    });
    let filter_session = query.session;

    // Spawn task to forward events to WebSocket
    let send_task = tokio::spawn(async move {
        loop {
            match events_rx.recv().await {
                Ok(event) => {
                    // Apply filters
                    if let Some(ref types) = filter_types {
                        let event_type = event_type_name(&event);
                        if !types.iter().any(|t| event_type.contains(t)) {
                            continue;
                        }
                    }

                    if let Some(ref session) = filter_session {
                        if !event_matches_session(&event, session) {
                            continue;
                        }
                    }

                    // Serialize and send
                    match serde_json::to_string(&event) {
                        Ok(json) => {
                            if sender.send(Message::Text(json)).await.is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to serialize event: {}", e);
                        }
                    }
                }
                Err(broadcast::error::RecvError::Closed) => break,
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("Firehose client lagged by {} events", n);
                }
            }
        }
    });

    // Handle incoming messages (for pause/resume commands)
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Close(_) => break,
            Message::Ping(data) => {
                // Pong is handled automatically by axum
                let _ = data;
            }
            _ => {}
        }
    }

    send_task.abort();
}

fn event_type_name(event: &VibesEvent) -> &'static str {
    match event {
        VibesEvent::SessionCreated { .. } => "SessionCreated",
        VibesEvent::SessionStateChanged { .. } => "SessionStateChanged",
        VibesEvent::SessionEnded { .. } => "SessionEnded",
        VibesEvent::Claude { .. } => "Claude",
        VibesEvent::Hook { .. } => "Hook",
        VibesEvent::Error { .. } => "Error",
    }
}

fn event_matches_session(event: &VibesEvent, session: &str) -> bool {
    match event {
        VibesEvent::SessionCreated { session_id, .. } => session_id == session,
        VibesEvent::SessionStateChanged { session_id, .. } => session_id == session,
        VibesEvent::SessionEnded { session_id, .. } => session_id == session,
        VibesEvent::Claude { session_id, .. } => session_id == session,
        VibesEvent::Hook { session_id, .. } => session_id == session,
        VibesEvent::Error { session_id, .. } => session_id.as_deref() == Some(session),
    }
}
```

**Step 2: Update handlers/mod.rs**

Add to exports:
```rust
mod firehose;
pub use firehose::firehose_ws;
```

**Step 3: Add route in lib.rs**

In the router setup, add:
```rust
.route("/ws/firehose", get(handlers::firehose_ws))
```

**Step 4: Run tests**

Run: `cargo test -p vibes-server`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-server/src/handlers
git commit -m "feat(server): add /ws/firehose WebSocket endpoint"
```

---

### Task 4.2: Create useFirehose hook

**Files:**
- Create: `web-ui/src/hooks/useFirehose.ts`
- Modify: `web-ui/src/hooks/index.ts`

**Step 1: Create useFirehose.ts**

```typescript
// web-ui/src/hooks/useFirehose.ts
import { useCallback, useEffect, useRef, useState } from 'react';
import { VibesEvent } from '../lib/types';

export interface FirehoseOptions {
  /** Event types to filter (e.g., ['Claude', 'Hook']) */
  types?: string[];
  /** Session ID to filter */
  session?: string;
  /** Maximum events to keep in buffer */
  bufferSize?: number;
  /** Auto-connect on mount */
  autoConnect?: boolean;
}

export interface UseFirehoseReturn {
  events: VibesEvent[];
  isConnected: boolean;
  isPaused: boolean;
  error: Error | null;
  connect: () => void;
  disconnect: () => void;
  pause: () => void;
  resume: () => void;
  clear: () => void;
}

export function useFirehose(options: FirehoseOptions = {}): UseFirehoseReturn {
  const { types, session, bufferSize = 1000, autoConnect = true } = options;

  const [events, setEvents] = useState<VibesEvent[]>([]);
  const [isConnected, setIsConnected] = useState(false);
  const [isPaused, setIsPaused] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const wsRef = useRef<WebSocket | null>(null);
  const bufferRef = useRef<VibesEvent[]>([]);

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return;

    const params = new URLSearchParams();
    if (types?.length) params.set('types', types.join(','));
    if (session) params.set('session', session);

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const url = `${protocol}//${window.location.host}/ws/firehose?${params}`;

    const ws = new WebSocket(url);
    wsRef.current = ws;

    ws.onopen = () => {
      setIsConnected(true);
      setError(null);
    };

    ws.onclose = () => {
      setIsConnected(false);
    };

    ws.onerror = () => {
      setError(new Error('WebSocket connection failed'));
    };

    ws.onmessage = (event) => {
      try {
        const vibesEvent = JSON.parse(event.data) as VibesEvent;

        // Always buffer (for resume)
        bufferRef.current = [...bufferRef.current.slice(-(bufferSize - 1)), vibesEvent];

        // Only update state if not paused
        if (!isPaused) {
          setEvents((prev) => [...prev.slice(-(bufferSize - 1)), vibesEvent]);
        }
      } catch (e) {
        console.error('Failed to parse firehose event:', e);
      }
    };
  }, [types, session, bufferSize, isPaused]);

  const disconnect = useCallback(() => {
    wsRef.current?.close();
    wsRef.current = null;
    setIsConnected(false);
  }, []);

  const pause = useCallback(() => {
    setIsPaused(true);
  }, []);

  const resume = useCallback(() => {
    // Sync buffer to events on resume
    setEvents(bufferRef.current);
    setIsPaused(false);
  }, []);

  const clear = useCallback(() => {
    setEvents([]);
    bufferRef.current = [];
  }, []);

  // Auto-connect
  useEffect(() => {
    if (autoConnect) {
      connect();
    }
    return () => {
      wsRef.current?.close();
    };
  }, [autoConnect, connect]);

  return {
    events,
    isConnected,
    isPaused,
    error,
    connect,
    disconnect,
    pause,
    resume,
    clear,
  };
}
```

**Step 2: Update hooks/index.ts**

Add export:
```typescript
export { useFirehose } from './useFirehose';
export type { FirehoseOptions, UseFirehoseReturn } from './useFirehose';
```

**Step 3: Commit**

```bash
git add web-ui/src/hooks/useFirehose.ts web-ui/src/hooks/index.ts
git commit -m "feat(web-ui): add useFirehose hook for event streaming"
```

---

## Phase 5: Migrate web-ui

### Task 5.1: Add design-system dependency to web-ui

**Files:**
- Modify: `web-ui/package.json`
- Modify: `web-ui/src/index.css`

**Step 1: Update package.json**

Add dependency:
```json
{
  "dependencies": {
    "@vibes/design-system": "*",
    // ... existing deps
  }
}
```

**Step 2: Replace index.css imports**

Replace the entire `web-ui/src/index.css` with:

```css
/* web-ui/src/index.css */
@import '@vibes/design-system/tokens';

/* App-specific overrides */
html, body, #root {
  height: 100%;
}

body {
  font-family: var(--font-sans);
  font-size: var(--text-base);
  line-height: var(--leading-normal);
  color: var(--color-text-primary);
  background-color: var(--color-bg-base);
}

.app {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.main {
  flex: 1;
  overflow: auto;
  padding: var(--space-6);
}

/* Keep existing terminal styles for xterm.js integration */
/* These will be migrated to design-system Terminal component later */
```

**Step 3: Install dependencies**

Run: `npm install`

**Step 4: Build and verify**

Run: `npm run build --workspace=web-ui`
Expected: Build succeeds

**Step 5: Commit**

```bash
git add web-ui/package.json web-ui/src/index.css
git commit -m "feat(web-ui): integrate @vibes/design-system tokens"
```

---

### Task 5.2: Update App.tsx with new Header

**Files:**
- Modify: `web-ui/src/App.tsx`

**Step 1: Update imports and RootLayout**

```tsx
// web-ui/src/App.tsx
import {
  createRouter,
  createRootRoute,
  createRoute,
  Outlet,
  Link,
  useLocation,
} from '@tanstack/react-router'
import { Header } from '@vibes/design-system'
import { ClaudeSessions } from './pages/ClaudeSessions'
import { ClaudeSession } from './pages/ClaudeSession'
import { StatusPage } from './pages/Status'
import { QuarantinePage } from './pages/Quarantine'
import { useAuth } from './hooks/useAuth'
import { useWebSocket } from './hooks/useWebSocket'
import { useState, useEffect } from 'react'

function RootLayout() {
  const { addMessageHandler } = useWebSocket();
  const { identity, isLocal, isAuthenticated } = useAuth({ addMessageHandler });
  const location = useLocation();
  const [theme, setTheme] = useState<'dark' | 'light'>('dark');

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
  }, [theme]);

  const navItems = [
    { label: 'Sessions', href: '/sessions', isActive: location.pathname.startsWith('/sessions') },
    { label: 'Streams', href: '/streams', isActive: location.pathname.startsWith('/streams') },
    { label: 'groove', href: '/groove', isGroove: true, isActive: location.pathname.startsWith('/groove') },
  ];

  return (
    <div className="app">
      <Header
        navItems={navItems}
        identity={isAuthenticated && identity ? { email: identity.email } : undefined}
        isLocal={isLocal}
        theme={theme}
        onThemeToggle={() => setTheme(t => t === 'dark' ? 'light' : 'dark')}
        renderLink={({ href, className, children }) => (
          <Link to={href} className={className}>{children}</Link>
        )}
      />
      <main className="main">
        <Outlet />
      </main>
    </div>
  );
}

// Update routes - rename /claude to /sessions
const claudeRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/sessions',
  component: ClaudeSessions,
});

const sessionRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/sessions/$sessionId',
  component: ClaudeSession,
});

// ... rest of routes
```

**Step 2: Build and verify**

Run: `npm run build --workspace=web-ui`
Expected: Build succeeds with new header

**Step 3: Commit**

```bash
git add web-ui/src/App.tsx
git commit -m "feat(web-ui): use design-system Header with theme toggle"
```

---

## Phase 6: Stream Views

### Task 6.1: Create StreamView composition

**Files:**
- Create: `design-system/src/compositions/StreamView/StreamView.tsx`
- Create: `design-system/src/compositions/StreamView/StreamView.module.css`
- Create: `design-system/src/compositions/StreamView/StreamView.test.tsx`
- Create: `design-system/src/compositions/StreamView/StreamView.stories.tsx`
- Create: `design-system/src/compositions/StreamView/index.ts`
- Modify: `design-system/src/compositions/index.ts`

(Implementation details follow the same TDD pattern as previous components)

---

### Task 6.2: Create EventInspector composition

(Similar structure to StreamView)

---

### Task 6.3: Create Firehose page

**Files:**
- Create: `web-ui/src/pages/Firehose.tsx`
- Modify: `web-ui/src/App.tsx`

---

### Task 6.4: Create Debug page

**Files:**
- Create: `web-ui/src/pages/Debug.tsx`
- Modify: `web-ui/src/App.tsx`

---

## Remaining Tasks

The following tasks follow the same TDD pattern:

- **Task 6.5**: Create Streams dashboard page
- **Task 6.6**: Update groove pages to use design-system
- **Task 6.7**: Add settings page with theme persistence
- **Task 6.8**: Add keyboard shortcuts (Cmd+K)
- **Task 6.9**: Responsive layout testing
- **Task 6.10**: Performance testing with 1000+ events

---

## Final Verification

**Step 1: Run all design-system tests**
```bash
cd design-system && npm test
```

**Step 2: Run Ladle and verify all stories**
```bash
cd design-system && npm run dev
```

**Step 3: Build web-ui**
```bash
npm run build --workspace=web-ui
```

**Step 4: Run e2e tests**
```bash
npm run test:e2e
```

**Step 5: Manual verification**
- Verify warm palette applied everywhere
- Test theme toggle
- Test firehose with live events
- Test debug view filters
- Test responsive breakpoints

**Step 6: Final commit**
```bash
git add .
git commit -m "feat: complete web UI modernization (Phase 4.5)"
```

---

## Summary

This plan implements the Web UI Modernization in 6 phases:

1. **Foundation** (Tasks 1.1-1.4): Set up design-system package with Ladle and tokens
2. **Primitives** (Tasks 2.1-2.6): Button, Badge, Panel, Text, StatusIndicator
3. **Compositions** (Tasks 3.1-3.2): SessionCard, Header
4. **Server** (Tasks 4.1-4.2): /ws/firehose endpoint, useFirehose hook
5. **Migration** (Tasks 5.1-5.2): Integrate design-system into web-ui
6. **Stream Views** (Tasks 6.1-6.10): StreamView, EventInspector, Firehose/Debug pages

Each task follows TDD: write failing test → implement → verify → commit.
