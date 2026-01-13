import { describe, test, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ModelsPage } from './Models';
import type { ModelInfo, CredentialInfo } from '../hooks/useModels';

// Mock TanStack Router
vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => vi.fn(),
  Link: ({ to, children }: { to: string; children: React.ReactNode }) => (
    <a href={to}>{children}</a>
  ),
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
        },
        {
          id: 'openai:gpt-4o',
          provider: 'openai',
          name: 'gpt-4o',
          context_window: 128000,
          capabilities: { chat: true, vision: true, tools: true, embeddings: false, streaming: true },
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
        },
        {
          id: 'openai:gpt-4o',
          provider: 'openai',
          name: 'gpt-4o',
          context_window: 128000,
          capabilities: { chat: true, vision: true, tools: true, embeddings: false, streaming: true },
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
});
