import '../../tokens/index.css';
import { Metric } from './Metric';

export default {
  title: 'Primitives/Metric',
};

export const Default = () => (
  <div style={{ padding: '2rem', backgroundColor: 'var(--screen)' }}>
    <Metric label="Success Rate" value="94.2%" />
  </div>
);

export const Sizes = () => (
  <div style={{ display: 'flex', gap: '2rem', padding: '2rem', backgroundColor: 'var(--screen)' }}>
    <Metric label="Small" value="42" size="sm" />
    <Metric label="Medium" value="42" size="md" />
    <Metric label="Large" value="42" size="lg" />
    <Metric label="Extra Large" value="42" size="xl" />
  </div>
);

export const DashboardExample = () => (
  <div style={{
    display: 'flex',
    gap: '2rem',
    padding: '2rem',
    backgroundColor: 'var(--surface)',
    border: '1px solid var(--border)'
  }}>
    <Metric label="Active Sessions" value={12} />
    <Metric label="Success Rate" value="94.2%" />
    <Metric label="Avg Duration" value="2.3m" />
  </div>
);

export const WithFormattedValues = () => (
  <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem', padding: '2rem', backgroundColor: 'var(--screen)' }}>
    <Metric label="Revenue" value="$12,345.67" />
    <Metric label="Growth" value="+15.3%" />
    <Metric label="Duration" value="2h 34m" />
    <Metric label="Count" value="1,234,567" />
  </div>
);

export const CustomContent = () => (
  <div style={{ padding: '2rem', backgroundColor: 'var(--screen)' }}>
    <Metric
      label="Status"
      value={
        <span style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <span style={{
            width: '10px',
            height: '10px',
            borderRadius: '50%',
            backgroundColor: 'var(--green)',
            boxShadow: '0 0 8px var(--green)'
          }} />
          Active
        </span>
      }
    />
  </div>
);
