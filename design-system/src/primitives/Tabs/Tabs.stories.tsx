import { useState } from 'react';
import '../../tokens/index.css';
import { Tabs } from './Tabs';

export default {
  title: 'Primitives/Tabs',
};

export const Default = () => {
  const [value, setValue] = useState('distribution');

  return (
    <div style={{ padding: '2rem', backgroundColor: 'var(--screen)' }}>
      <Tabs value={value} onChange={setValue}>
        <Tabs.Tab value="distribution">Distribution</Tabs.Tab>
        <Tabs.Tab value="overrides">Overrides</Tabs.Tab>
      </Tabs>
      <div style={{ padding: '1rem', color: 'var(--text)' }}>
        Active tab: {value}
      </div>
    </div>
  );
};

export const ThreeTabs = () => {
  const [value, setValue] = useState('leaderboard');

  return (
    <div style={{ padding: '2rem', backgroundColor: 'var(--screen)' }}>
      <Tabs value={value} onChange={setValue}>
        <Tabs.Tab value="leaderboard">Leaderboard</Tabs.Tab>
        <Tabs.Tab value="timeline">Timeline</Tabs.Tab>
        <Tabs.Tab value="ablation">Ablation</Tabs.Tab>
      </Tabs>
      <div style={{ padding: '1rem', color: 'var(--text)' }}>
        Active tab: {value}
      </div>
    </div>
  );
};

export const WithDisabled = () => {
  const [value, setValue] = useState('active');

  return (
    <div style={{ padding: '2rem', backgroundColor: 'var(--screen)' }}>
      <Tabs value={value} onChange={setValue}>
        <Tabs.Tab value="active">Active</Tabs.Tab>
        <Tabs.Tab value="disabled" disabled>Disabled</Tabs.Tab>
        <Tabs.Tab value="another">Another</Tabs.Tab>
      </Tabs>
    </div>
  );
};

export const ManyTabs = () => {
  const [value, setValue] = useState('overview');

  return (
    <div style={{ padding: '2rem', backgroundColor: 'var(--screen)' }}>
      <Tabs value={value} onChange={setValue}>
        <Tabs.Tab value="overview">Overview</Tabs.Tab>
        <Tabs.Tab value="learnings">Learnings</Tabs.Tab>
        <Tabs.Tab value="attribution">Attribution</Tabs.Tab>
        <Tabs.Tab value="strategy">Strategy</Tabs.Tab>
        <Tabs.Tab value="health">Health</Tabs.Tab>
      </Tabs>
    </div>
  );
};
