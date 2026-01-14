import type { StoryDefault, Story } from '@ladle/react';
import { EmptyState } from './EmptyState';
import { Button } from '../Button';

export default {
  title: 'Primitives/EmptyState',
} satisfies StoryDefault;

export const Default: Story = () => (
  <EmptyState message="No items found" />
);

export const WithHint: Story = () => (
  <EmptyState
    message="No models registered"
    hint="Register a provider to see available models."
  />
);

export const WithIcon: Story = () => (
  <EmptyState
    icon="📦"
    message="No items found"
    hint="Your collection is empty"
  />
);

export const WithAction: Story = () => (
  <EmptyState
    icon="🔍"
    message="No results"
    hint="Try adjusting your search criteria"
    action={<Button size="sm">Clear filters</Button>}
  />
);

export const SmallSize: Story = () => (
  <EmptyState
    size="sm"
    message="No data"
    hint="Compact empty state"
  />
);

export const LargeSize: Story = () => (
  <EmptyState
    size="lg"
    icon="🌌"
    message="Nothing here yet"
    hint="This space is waiting to be filled"
    action={<Button>Get started</Button>}
  />
);

export const InCard: Story = () => (
  <div style={{ maxWidth: 400, background: 'var(--surface)', border: '1px solid var(--border)', borderRadius: 'var(--radius-md)' }}>
    <EmptyState
      message="No sessions"
      hint="Sessions will appear here once you start working"
    />
  </div>
);
