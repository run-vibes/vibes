import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { TerminalPanel } from './TerminalPanel';
import { TerminalHeader } from './TerminalHeader';
import { TerminalInput } from './TerminalInput';
import { TerminalLine } from './TerminalLine';

describe('TerminalPanel', () => {
  it('renders children in body slot', () => {
    render(<TerminalPanel>Body content</TerminalPanel>);
    expect(screen.getByText('Body content')).toBeInTheDocument();
  });

  it('renders header slot when provided', () => {
    render(<TerminalPanel header={<div>Header content</div>}>Body</TerminalPanel>);
    expect(screen.getByText('Header content')).toBeInTheDocument();
  });

  it('renders input slot when provided', () => {
    render(<TerminalPanel input={<div>Input content</div>}>Body</TerminalPanel>);
    expect(screen.getByText('Input content')).toBeInTheDocument();
  });

  it('applies focused class when focused', () => {
    const { container } = render(<TerminalPanel focused>Body</TerminalPanel>);
    expect(container.firstChild?.className).toMatch(/focused/);
  });

  it('merges custom className', () => {
    const { container } = render(<TerminalPanel className="custom">Body</TerminalPanel>);
    expect(container.firstChild).toHaveClass('custom');
  });
});

describe('TerminalHeader', () => {
  it('renders name when provided', () => {
    render(<TerminalHeader name="api-refactor" />);
    expect(screen.getByText('api-refactor')).toBeInTheDocument();
  });

  it('renders id when provided', () => {
    render(<TerminalHeader id="a7f3c2" />);
    expect(screen.getByText('a7f3c2')).toBeInTheDocument();
  });

  it('renders metadata items', () => {
    render(<TerminalHeader metadata={['47 tools', '1h 23m']} />);
    expect(screen.getByText('47 tools')).toBeInTheDocument();
    expect(screen.getByText('1h 23m')).toBeInTheDocument();
  });

  it('renders actions when provided', () => {
    render(<TerminalHeader actions={<button>PAUSE</button>} />);
    expect(screen.getByRole('button', { name: 'PAUSE' })).toBeInTheDocument();
  });

  it('applies status class for active status', () => {
    const { container } = render(<TerminalHeader status="active" />);
    const statusDot = container.querySelector('[class*="status"]');
    expect(statusDot?.className).toMatch(/active/);
  });

  it('applies status class for error status', () => {
    const { container } = render(<TerminalHeader status="error" />);
    const statusDot = container.querySelector('[class*="status"]');
    expect(statusDot?.className).toMatch(/error/);
  });
});

describe('TerminalInput', () => {
  it('renders default prompt character', () => {
    render(<TerminalInput />);
    expect(screen.getByText('⟩')).toBeInTheDocument();
  });

  it('renders custom prompt character', () => {
    render(<TerminalInput prompt="$" />);
    expect(screen.getByText('$')).toBeInTheDocument();
  });

  it('renders input field with placeholder', () => {
    render(<TerminalInput placeholder="Send a message..." />);
    expect(screen.getByPlaceholderText('Send a message...')).toBeInTheDocument();
  });

  it('shows cursor when showCursor is true', () => {
    const { container } = render(<TerminalInput showCursor />);
    expect(container.querySelector('[class*="cursor"]')).toBeInTheDocument();
  });

  it('calls onSubmit when Enter is pressed', () => {
    const onSubmit = vi.fn();
    render(<TerminalInput onSubmit={onSubmit} />);
    const input = screen.getByRole('textbox');
    fireEvent.change(input, { target: { value: 'test message' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(onSubmit).toHaveBeenCalledWith('test message');
  });
});

describe('TerminalLine', () => {
  it('renders prompt variant with prompt text', () => {
    render(<TerminalLine variant="prompt" prompt="~/vibes $">ls -la</TerminalLine>);
    expect(screen.getByText('~/vibes $')).toBeInTheDocument();
    expect(screen.getByText('ls -la')).toBeInTheDocument();
  });

  it('renders thinking variant with icon', () => {
    render(<TerminalLine variant="thinking">Analyzing code...</TerminalLine>);
    expect(screen.getByText('🧠')).toBeInTheDocument();
    expect(screen.getByText('Analyzing code...')).toBeInTheDocument();
  });

  it('renders tool variant with tool name', () => {
    render(<TerminalLine variant="tool" toolName="Read">file.txt</TerminalLine>);
    expect(screen.getByText('Read')).toBeInTheDocument();
    expect(screen.getByText('file.txt')).toBeInTheDocument();
  });

  it('renders output variant with success status', () => {
    const { container } = render(
      <TerminalLine variant="output" status="success">All tests passed</TerminalLine>
    );
    expect(container.firstChild?.className).toMatch(/success/);
    expect(screen.getByText('All tests passed')).toBeInTheDocument();
  });

  it('renders output variant with error status', () => {
    const { container } = render(
      <TerminalLine variant="output" status="error">Build failed</TerminalLine>
    );
    expect(container.firstChild?.className).toMatch(/error/);
  });

  it('renders output variant with info status', () => {
    const { container } = render(
      <TerminalLine variant="output" status="info">Found 3 matches</TerminalLine>
    );
    expect(container.firstChild?.className).toMatch(/info/);
  });
});
