// design-system/src/compositions/EventInspector/EventInspector.test.tsx
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { EventInspector } from './EventInspector';

describe('EventInspector', () => {
  const mockEvent = {
    id: 'evt-7f3a8b2c-1234',
    timestamp: new Date('2024-12-31T14:32:03.012Z'),
    type: 'ERROR',
    session: 'sess-abc',
    sessionName: 'auth-refactor',
    payload: {
      error: 'PermissionDenied',
      path: '/etc/passwd',
      operation: 'read',
    },
  };

  it('renders event ID', () => {
    render(<EventInspector event={mockEvent} />);
    expect(screen.getByText(/evt-7f3a8b2c-1234/)).toBeInTheDocument();
  });

  it('renders event type', () => {
    render(<EventInspector event={mockEvent} />);
    expect(screen.getByText(/ERROR/)).toBeInTheDocument();
  });

  it('renders timestamp', () => {
    render(<EventInspector event={mockEvent} />);
    expect(screen.getByText(/14:32:03/)).toBeInTheDocument();
  });

  it('renders session info', () => {
    render(<EventInspector event={mockEvent} />);
    expect(screen.getByText(/sess-abc/)).toBeInTheDocument();
    expect(screen.getByText(/auth-refactor/)).toBeInTheDocument();
  });

  it('renders payload as JSON', () => {
    render(<EventInspector event={mockEvent} />);
    expect(screen.getByText(/PermissionDenied/)).toBeInTheDocument();
  });

  it('calls onCopyJson when copy button clicked', () => {
    const onCopy = vi.fn();
    render(<EventInspector event={mockEvent} onCopyJson={onCopy} />);
    fireEvent.click(screen.getByText(/Copy JSON/i));
    expect(onCopy).toHaveBeenCalled();
  });

  it('renders empty state when no event', () => {
    render(<EventInspector event={null} />);
    expect(screen.getByText(/No event selected/i)).toBeInTheDocument();
  });
});
