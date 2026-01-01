// design-system/src/compositions/StreamView/StreamView.test.tsx
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { StreamView } from './StreamView';

describe('StreamView', () => {
  const mockEvents = [
    { id: '1', timestamp: new Date('2024-12-31T14:32:01Z'), type: 'SESSION', session: 'sess-abc', summary: 'Created "auth-refactor"' },
    { id: '2', timestamp: new Date('2024-12-31T14:32:02Z'), type: 'CLAUDE', session: 'sess-abc', summary: 'Let me analyze...' },
    { id: '3', timestamp: new Date('2024-12-31T14:32:03Z'), type: 'TOOL', session: 'sess-abc', summary: 'Read src/lib.rs' },
    { id: '4', timestamp: new Date('2024-12-31T14:32:04Z'), type: 'ERROR', session: 'sess-abc', summary: 'Permission denied' },
  ];

  it('renders events', () => {
    render(<StreamView events={mockEvents} />);
    expect(screen.getByText(/Created "auth-refactor"/)).toBeInTheDocument();
    expect(screen.getByText(/Let me analyze/)).toBeInTheDocument();
  });

  it('renders empty state when no events', () => {
    render(<StreamView events={[]} />);
    expect(screen.getByText(/No events/i)).toBeInTheDocument();
  });

  it('applies event type classes', () => {
    render(<StreamView events={mockEvents} />);
    const errorEvent = screen.getByText(/Permission denied/).closest('[class*="event"]');
    expect(errorEvent?.className).toMatch(/error/);
  });

  it('calls onEventClick when event is clicked', () => {
    const onClick = vi.fn();
    render(<StreamView events={mockEvents} onEventClick={onClick} />);
    fireEvent.click(screen.getByText(/Created "auth-refactor"/));
    expect(onClick).toHaveBeenCalledWith(mockEvents[0]);
  });

  it('shows live indicator when isLive', () => {
    render(<StreamView events={mockEvents} isLive />);
    expect(screen.getByText(/LIVE/i)).toBeInTheDocument();
  });

  it('shows paused indicator when isPaused', () => {
    render(<StreamView events={mockEvents} isPaused />);
    expect(screen.getByText(/PAUSED/i)).toBeInTheDocument();
  });
});
