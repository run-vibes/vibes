import { useState, useCallback } from 'react';

export interface ModelInfo {
  id: string;
  provider: string;
  name: string;
  context_window: number;
  capabilities: {
    chat: boolean;
    vision: boolean;
    tools: boolean;
    embeddings: boolean;
    streaming: boolean;
  };
  pricing?: {
    input_per_million: number;
    output_per_million: number;
  };
  local?: boolean;
}

export interface CredentialInfo {
  provider: string;
  source: 'keyring' | 'environment';
}

interface UseModelsReturn {
  models: ModelInfo[];
  providers: string[];
  credentials: CredentialInfo[];
  isLoading: boolean;
  error: string | null;
  refresh: () => void;
}

/**
 * Hook for managing models and credentials.
 * Currently returns empty data - will be connected to WebSocket API.
 */
export function useModels(): UseModelsReturn {
  const [models] = useState<ModelInfo[]>([]);
  const [providers] = useState<string[]>([]);
  const [credentials] = useState<CredentialInfo[]>([]);
  const [isLoading] = useState(false);
  const [error] = useState<string | null>(null);

  const refresh = useCallback(() => {
    // TODO: Send WebSocket message to refresh models/credentials
  }, []);

  return {
    models,
    providers,
    credentials,
    isLoading,
    error,
    refresh,
  };
}
