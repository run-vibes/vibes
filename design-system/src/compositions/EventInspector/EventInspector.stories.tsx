// design-system/src/compositions/EventInspector/EventInspector.stories.tsx
import '../../tokens/index.css';
import { EventInspector } from './EventInspector';
import type { DisplayEvent, ContextEvent } from '../../events';

const mockEvent: DisplayEvent = {
  id: 'evt-7f3a8b2c-1234-5678-9abc-def012345678',
  timestamp: new Date('2024-12-31T14:32:03.012Z'),
  type: 'ERROR',
  summary: 'Permission denied: /etc/passwd',
  session: 'sess-abc',
  sessionName: 'auth-refactor',
  offset: 1247,
  payload: {
    error: 'PermissionDenied',
    path: '/etc/passwd',
    operation: 'read',
  },
};

const mockContext: ContextEvent[] = [
  { relativePosition: -2, timestamp: new Date('2024-12-31T14:32:02.901Z'), type: 'TOOL', summary: 'Read src/auth.rs' },
  { relativePosition: -1, timestamp: new Date('2024-12-31T14:32:02.998Z'), type: 'CLAUDE', summary: 'Now let me check...' },
  { relativePosition: 0, timestamp: new Date('2024-12-31T14:32:03.012Z'), type: 'ERROR', summary: 'Permission denied' },
  { relativePosition: 1, timestamp: new Date('2024-12-31T14:32:03.234Z'), type: 'CLAUDE', summary: 'I see there was an error' },
];

export const Default = () => (
  <div style={{ height: '600px', padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <EventInspector
      event={mockEvent}
      contextEvents={mockContext}
      onCopyJson={() => alert('Copied!')}
    />
  </div>
);

export const WithoutContext = () => (
  <div style={{ height: '400px', padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <EventInspector event={mockEvent} onCopyJson={() => alert('Copied!')} />
  </div>
);

export const ToolEvent = () => (
  <div style={{ height: '400px', padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <EventInspector
      event={{
        id: 'evt-tool-1234',
        timestamp: new Date(),
        type: 'TOOL',
        summary: 'Read src/lib.rs (2.1kb)',
        session: 'sess-xyz',
        payload: {
          tool: 'Read',
          path: 'src/lib.rs',
          bytes: 2148,
        },
      }}
      onCopyJson={() => alert('Copied!')}
    />
  </div>
);

export const Empty = () => (
  <div style={{ height: '300px', padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <EventInspector event={null} />
  </div>
);
