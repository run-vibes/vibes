import '../../tokens/index.css';
import { Badge } from './Badge';

export default {
  title: 'Primitives/Badge',
};

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
