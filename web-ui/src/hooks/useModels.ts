import { useCallback, useEffect, useState } from 'react';
import type { ClientMessage, ServerMessage, ModelInfo } from '../lib/types';

// Re-export ModelInfo for consumers
export type { ModelInfo } from '../lib/types';

export interface CredentialInfo {
  provider: string;
  source: 'keyring' | 'environment';
}

function generateRequestId(): string {
  return `${Date.now()}-${Math.random().toString(36).slice(2, 11)}`;
}

interface UseModelsOptions {
  /** Function to send WebSocket messages */
  send: (message: ClientMessage) => void;
  /** Function to add a message handler */
  addMessageHandler: (handler: (message: ServerMessage) => void) => () => void;
  /** Whether the WebSocket is connected */
  isConnected: boolean;
  /** Auto-refresh on connect (default: true) */
  autoRefresh?: boolean;
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
 * Fetches model list from server via WebSocket.
 */
export function useModels(options: UseModelsOptions): UseModelsReturn {
  const { send, addMessageHandler, isConnected, autoRefresh = true } = options;

  const [models, setModels] = useState<ModelInfo[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pendingRequestId, setPendingRequestId] = useState<string | null>(null);

  // Derive providers from models
  const providers = [...new Set(models.map((m) => m.provider))];

  // Credentials not yet implemented - placeholder for future
  const credentials: CredentialInfo[] = [];

  // Refresh the model list
  const refresh = useCallback(() => {
    if (!isConnected) {
      setError('Not connected');
      return;
    }

    const requestId = generateRequestId();
    setPendingRequestId(requestId);
    setIsLoading(true);
    setError(null);

    send({ type: 'list_models', request_id: requestId });
  }, [isConnected, send]);

  // Handle incoming messages
  useEffect(() => {
    const cleanup = addMessageHandler((message: ServerMessage) => {
      switch (message.type) {
        case 'model_list':
          if (message.request_id === pendingRequestId) {
            setModels(message.models);
            setIsLoading(false);
            setPendingRequestId(null);
          }
          break;

        case 'error':
          if (pendingRequestId) {
            setError(message.message);
            setIsLoading(false);
            setPendingRequestId(null);
          }
          break;
      }
    });

    return cleanup;
  }, [addMessageHandler, pendingRequestId]);

  // Auto-refresh on connect
  useEffect(() => {
    if (isConnected && autoRefresh) {
      refresh();
    }
  }, [isConnected, autoRefresh, refresh]);

  return {
    models,
    providers,
    credentials,
    isLoading,
    error,
    refresh,
  };
}
