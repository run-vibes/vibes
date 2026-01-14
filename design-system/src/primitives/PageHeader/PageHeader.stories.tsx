import '../../tokens/index.css';
import { PageHeader } from './PageHeader';
import { Badge } from '../Badge';
import { Button } from '../Button';

export default {
  title: 'Primitives/PageHeader',
};

export const Default = () => (
  <div style={{ backgroundColor: 'var(--screen)', minHeight: '200px' }}>
    <PageHeader title="STATUS" />
  </div>
);

export const WithLeftContent = () => (
  <div style={{ backgroundColor: 'var(--screen)', minHeight: '200px' }}>
    <PageHeader
      title="STREAM"
      leftContent={
        <div style={{ display: 'flex', gap: 'var(--space-2)' }}>
          <Badge status="success">Connected</Badge>
          <Badge>LIVE</Badge>
        </div>
      }
    />
  </div>
);

export const WithRightContent = () => (
  <div style={{ backgroundColor: 'var(--screen)', minHeight: '200px' }}>
    <PageHeader
      title="LEARNINGS"
      rightContent={<Button size="sm">Export</Button>}
    />
  </div>
);

export const WithBothSides = () => (
  <div style={{ backgroundColor: 'var(--screen)', minHeight: '200px' }}>
    <PageHeader
      title="ASSESSMENT"
      leftContent={
        <div style={{ display: 'flex', gap: 'var(--space-2)' }}>
          <Badge status="success">Connected</Badge>
        </div>
      }
      rightContent={
        <div style={{ display: 'flex', gap: 'var(--space-2)' }}>
          <Button size="sm" variant="ghost">Pause</Button>
          <Button size="sm">Clear</Button>
        </div>
      }
    />
  </div>
);

export const AllPageHeaders = () => (
  <div style={{ backgroundColor: 'var(--screen)', display: 'flex', flexDirection: 'column', gap: 'var(--space-4)', padding: 'var(--space-4)' }}>
    <PageHeader title="STATUS" />
    <PageHeader title="LEARNINGS" />
    <PageHeader title="STRATEGY" />
    <PageHeader title="STREAM" leftContent={<Badge status="success">Connected</Badge>} />
    <PageHeader title="HISTORY" />
    <PageHeader title="OPENWORLD" />
  </div>
);
