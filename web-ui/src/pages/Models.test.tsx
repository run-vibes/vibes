import { describe, test, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { ModelsPage } from './Models';
import type { ModelInfo } from '../lib/types';
import type { CredentialInfo } from '../hooks/useModels';

// Mock TanStack Router
vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => vi.fn(),
  Link: ({ to, children }: { to: string; children: React.ReactNode }) => (
    <a href={to}>{children}</a>
  ),
}));

// Mock useWebSocket hook
vi.mock('../hooks/useWebSocket', () => ({
  useWebSocket: () => ({
    send: vi.fn(),
    addMessageHandler: vi.fn(() => vi.fn()),
    isConnected: true,
    connectionState: 'connected',
  }),
}));

interface MockUseModelsReturn {
  models: ModelInfo[];
  providers: string[];
  credentials: CredentialInfo[];
  isLoading: boolean;
  error: string | null;
  refresh: () => void;
}

// Mock useModels hook - default empty state
const mockUseModels = vi.fn<[], MockUseModelsReturn>(() => ({
  models: [],
  providers: [],
  credentials: [],
  isLoading: false,
  error: null,
  refresh: vi.fn(),
}));

vi.mock('../hooks/useModels', () => ({
  useModels: () => mockUseModels(),
}));

describe('ModelsPage', () => {
  beforeEach(() => {
    mockUseModels.mockReturnValue({
      models: [],
      providers: [],
      credentials: [],
      isLoading: false,
      error: null,
      refresh: vi.fn(),
    });
  });

  test('renders Models heading', () => {
    render(<ModelsPage />);
    expect(screen.getByRole('heading', { name: /models/i })).toBeInTheDocument();
  });

  test('shows empty state when no models', () => {
    render(<ModelsPage />);
    expect(screen.getByText(/no models registered/i)).toBeInTheDocument();
  });

  test('displays models in table when models exist', () => {
    mockUseModels.mockReturnValue({
      models: [
        {
          id: 'anthropic:claude-sonnet-4',
          provider: 'anthropic',
          name: 'claude-sonnet-4',
          context_window: 200000,
          capabilities: { chat: true, vision: true, tools: true, embeddings: false, streaming: true },
          local: false,
        },
        {
          id: 'openai:gpt-4o',
          provider: 'openai',
          name: 'gpt-4o',
          context_window: 128000,
          capabilities: { chat: true, vision: true, tools: true, embeddings: false, streaming: true },
          local: false,
        },
      ],
      providers: ['anthropic', 'openai'],
      credentials: [{ provider: 'anthropic', source: 'keyring' }],
      isLoading: false,
      error: null,
      refresh: vi.fn(),
    });

    render(<ModelsPage />);

    // Should not show empty state
    expect(screen.queryByText(/no models registered/i)).not.toBeInTheDocument();

    // Should show model names
    expect(screen.getByText('claude-sonnet-4')).toBeInTheDocument();
    expect(screen.getByText('gpt-4o')).toBeInTheDocument();

    // Should show providers in table (may also appear in filter dropdown)
    expect(screen.getAllByText('anthropic').length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText('openai').length).toBeGreaterThanOrEqual(1);
  });

  test('shows provider filter dropdown', () => {
    mockUseModels.mockReturnValue({
      models: [
        {
          id: 'anthropic:claude-sonnet-4',
          provider: 'anthropic',
          name: 'claude-sonnet-4',
          context_window: 200000,
          capabilities: { chat: true, vision: true, tools: true, embeddings: false, streaming: true },
          local: false,
        },
      ],
      providers: ['anthropic', 'openai'],
      credentials: [],
      isLoading: false,
      error: null,
      refresh: vi.fn(),
    });

    render(<ModelsPage />);

    // Should show a filter dropdown
    expect(screen.getByLabelText(/provider/i)).toBeInTheDocument();
  });

  test('shows authenticated indicator for providers with credentials', () => {
    mockUseModels.mockReturnValue({
      models: [
        {
          id: 'anthropic:claude-sonnet-4',
          provider: 'anthropic',
          name: 'claude-sonnet-4',
          context_window: 200000,
          capabilities: { chat: true, vision: true, tools: true, embeddings: false, streaming: true },
          local: false,
        },
        {
          id: 'openai:gpt-4o',
          provider: 'openai',
          name: 'gpt-4o',
          context_window: 128000,
          capabilities: { chat: true, vision: true, tools: true, embeddings: false, streaming: true },
          local: false,
        },
      ],
      providers: ['anthropic', 'openai'],
      credentials: [{ provider: 'anthropic', source: 'keyring' }],
      isLoading: false,
      error: null,
      refresh: vi.fn(),
    });

    render(<ModelsPage />);

    // Should show authenticated indicator for anthropic (has credentials)
    expect(screen.getByTitle('Authenticated via keyring')).toBeInTheDocument();
  });

  test('filters models by capability', () => {
    mockUseModels.mockReturnValue({
      models: [
        {
          id: 'anthropic:claude-sonnet-4',
          provider: 'anthropic',
          name: 'claude-sonnet-4',
          context_window: 200000,
          capabilities: { chat: true, vision: true, tools: true, embeddings: false, streaming: true },
          local: false,
        },
        {
          id: 'openai:text-embedding-3',
          provider: 'openai',
          name: 'text-embedding-3',
          context_window: 8191,
          capabilities: { chat: false, vision: false, tools: false, embeddings: true, streaming: false },
          local: false,
        },
      ],
      providers: ['anthropic', 'openai'],
      credentials: [],
      isLoading: false,
      error: null,
      refresh: vi.fn(),
    });

    render(<ModelsPage />);

    // Both models should be visible initially
    expect(screen.getByText('claude-sonnet-4')).toBeInTheDocument();
    expect(screen.getByText('text-embedding-3')).toBeInTheDocument();

    // Filter by embeddings capability
    const capabilityFilter = screen.getByLabelText(/capability/i);
    fireEvent.change(capabilityFilter, { target: { value: 'embeddings' } });

    // Only embedding model should be visible
    expect(screen.queryByText('claude-sonnet-4')).not.toBeInTheDocument();
    expect(screen.getByText('text-embedding-3')).toBeInTheDocument();
  });

  test('shows model details panel when clicking a row', () => {
    mockUseModels.mockReturnValue({
      models: [
        {
          id: 'anthropic:claude-sonnet-4',
          provider: 'anthropic',
          name: 'claude-sonnet-4',
          context_window: 200000,
          capabilities: { chat: true, vision: true, tools: true, embeddings: false, streaming: true },
          pricing: { input_per_million: 3, output_per_million: 15 },
          local: false,
        },
      ],
      providers: ['anthropic'],
      credentials: [{ provider: 'anthropic', source: 'keyring' }],
      isLoading: false,
      error: null,
      refresh: vi.fn(),
    });

    render(<ModelsPage />);

    // Click on the model row
    const row = screen.getByText('claude-sonnet-4').closest('tr');
    expect(row).toBeInTheDocument();
    fireEvent.click(row!);

    // Details panel should appear with model info
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(screen.getByText(/\$3\/M input/i)).toBeInTheDocument();
  });

  test('displays size and modified date for local models', () => {
    mockUseModels.mockReturnValue({
      models: [
        {
          id: 'ollama:llama3',
          provider: 'ollama',
          name: 'llama3',
          context_window: 8192,
          capabilities: { chat: true, vision: false, tools: false, embeddings: false, streaming: true },
          local: true,
          size_bytes: 4_500_000_000, // 4.5 GB
          modified_at: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString(), // 2 days ago
        },
      ],
      providers: ['ollama'],
      credentials: [],
      isLoading: false,
      error: null,
      refresh: vi.fn(),
    });

    render(<ModelsPage />);

    // Should show size
    expect(screen.getByText('4.2 GB')).toBeInTheDocument();

    // Should show relative date
    expect(screen.getByText('2 days ago')).toBeInTheDocument();
  });
});
