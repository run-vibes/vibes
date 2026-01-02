// design-system/src/compositions/StreamView/StreamView.stories.tsx
import { useState } from 'react';
import '../../tokens/index.css';
import { StreamView } from './StreamView';
import type { DisplayEvent } from '../../events';

const mockEvents: DisplayEvent[] = [
  { id: '1', timestamp: new Date(Date.now() - 60000), type: 'SESSION', session: 'sess-abc', summary: 'Created "auth-refactor"' },
  { id: '2', timestamp: new Date(Date.now() - 55000), type: 'CLAUDE', session: 'sess-abc', summary: 'Let me analyze the authentication flow...' },
  { id: '3', timestamp: new Date(Date.now() - 50000), type: 'TOOL', session: 'sess-abc', summary: 'Read src/lib.rs (2.1kb)' },
  { id: '4', timestamp: new Date(Date.now() - 45000), type: 'ASSESS', session: 'sess-abc', summary: 'Lightweight: OK' },
  { id: '5', timestamp: new Date(Date.now() - 40000), type: 'CLAUDE', session: 'sess-abc', summary: 'I see the current implementation uses...' },
  { id: '6', timestamp: new Date(Date.now() - 35000), type: 'TOOL', session: 'sess-abc', summary: 'Edit src/auth.rs:47-52' },
  { id: '7', timestamp: new Date(Date.now() - 30000), type: 'HOOK', session: 'sess-abc', summary: 'ToolResult: success' },
  { id: '8', timestamp: new Date(Date.now() - 25000), type: 'ERROR', session: 'sess-abc', summary: 'Permission denied: /etc/passwd' },
];

export const Default = () => (
  <div style={{ height: '400px', padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <StreamView events={mockEvents} title="Firehose" />
  </div>
);

export const LiveStreaming = () => (
  <div style={{ height: '400px', padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <StreamView events={mockEvents} title="Firehose" isLive />
  </div>
);

export const Paused = () => (
  <div style={{ height: '400px', padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <StreamView events={mockEvents} title="Firehose" isLive isPaused />
  </div>
);

export const WithSelection = () => {
  const [selected, setSelected] = useState<string | undefined>('3');
  return (
    <div style={{ height: '400px', padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
      <StreamView
        events={mockEvents}
        title="Firehose"
        selectedId={selected}
        onEventClick={(e) => setSelected(e.id)}
      />
    </div>
  );
};

export const Empty = () => (
  <div style={{ height: '400px', padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <StreamView events={[]} title="Firehose" />
  </div>
);
