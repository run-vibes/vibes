import '../../tokens/index.css';
import { Button } from './Button';

export default {
  title: 'Primitives/Button',
};

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

export const AllCombinations = () => (
  <div style={{ padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <div style={{ display: 'flex', gap: '1rem', marginBottom: '1rem' }}>
      <Button variant="primary" size="sm">Primary SM</Button>
      <Button variant="primary" size="md">Primary MD</Button>
      <Button variant="primary" size="lg">Primary LG</Button>
    </div>
    <div style={{ display: 'flex', gap: '1rem', marginBottom: '1rem' }}>
      <Button variant="secondary" size="sm">Secondary SM</Button>
      <Button variant="secondary" size="md">Secondary MD</Button>
      <Button variant="secondary" size="lg">Secondary LG</Button>
    </div>
    <div style={{ display: 'flex', gap: '1rem' }}>
      <Button variant="ghost" size="sm">Ghost SM</Button>
      <Button variant="ghost" size="md">Ghost MD</Button>
      <Button variant="ghost" size="lg">Ghost LG</Button>
    </div>
  </div>
);
