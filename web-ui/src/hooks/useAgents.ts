import { useCallback, useEffect, useState } from 'react';
import type { AgentInfo, AgentType, ClientMessage, ServerMessage } from '../lib/types';

function generateRequestId(): string {
  return `${Date.now()}-${Math.random().toString(36).slice(2, 11)}`;
}

interface UseAgentsOptions {
  /** Function to send WebSocket messages */
  send: (message: ClientMessage) => void;
  /** Function to add a message handler */
  addMessageHandler: (handler: (message: ServerMessage) => void) => () => void;
  /** Whether the WebSocket is connected */
  isConnected: boolean;
  /** Auto-refresh on connect (default: true) */
  autoRefresh?: boolean;
  /** Refresh interval in ms (default: 0 = no auto-refresh) */
  refreshInterval?: number;
}

interface UseAgentsReturn {
  /** List of agents */
  agents: AgentInfo[];
  /** Whether the list is currently loading */
  isLoading: boolean;
  /** Whether an agent is being spawned */
  isSpawning: boolean;
  /** Error message if operation failed */
  error: string | null;
  /** Refresh the agent list */
  refresh: () => void;
  /** Spawn a new agent, returns the agent */
  spawnAgent: (type: AgentType, name?: string, task?: string) => Promise<AgentInfo>;
  /** Pause an agent */
  pauseAgent: (agentId: string) => Promise<void>;
  /** Resume an agent */
  resumeAgent: (agentId: string) => Promise<void>;
  /** Cancel an agent's current task */
  cancelAgent: (agentId: string) => Promise<void>;
  /** Stop and remove an agent */
  stopAgent: (agentId: string) => Promise<void>;
  /** Get agent status */
  getAgentStatus: (agentId: string) => Promise<AgentInfo>;
}

/**
 * Hook for managing agents
 */
export function useAgents(options: UseAgentsOptions): UseAgentsReturn {
  const {
    send,
    addMessageHandler,
    isConnected,
    autoRefresh = true,
    refreshInterval = 0,
  } = options;

  const [agents, setAgents] = useState<AgentInfo[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isSpawning, setIsSpawning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pendingRequestId, setPendingRequestId] = useState<string | null>(null);

  // For spawn operation
  const [spawnResolver, setSpawnResolver] = useState<{
    requestId: string;
    resolve: (agent: AgentInfo) => void;
    reject: (error: Error) => void;
  } | null>(null);

  // For status/pause/resume/cancel/stop operations
  const [pendingOperations, setPendingOperations] = useState<Map<string, {
    resolve: (value: unknown) => void;
    reject: (error: Error) => void;
  }>>(new Map());

  // Refresh the agent list
  const refresh = useCallback(() => {
    if (!isConnected) {
      setError('Not connected');
      return;
    }

    const requestId = generateRequestId();
    setPendingRequestId(requestId);
    setIsLoading(true);
    setError(null);

    send({ type: 'list_agents', request_id: requestId });
  }, [isConnected, send]);

  // Spawn a new agent
  const spawnAgent = useCallback(
    (type: AgentType, name?: string, task?: string): Promise<AgentInfo> => {
      return new Promise((resolve, reject) => {
        if (!isConnected) {
          reject(new Error('Not connected'));
          return;
        }

        const requestId = generateRequestId();
        setIsSpawning(true);
        setError(null);
        setSpawnResolver({ requestId, resolve, reject });

        send({ type: 'spawn_agent', request_id: requestId, agent_type: type, name, task });
      });
    },
    [isConnected, send]
  );

  // Generic operation helper
  const performOperation = useCallback(
    <T>(
      messageType: 'pause_agent' | 'resume_agent' | 'cancel_agent' | 'stop_agent' | 'agent_status',
      agentId: string
    ): Promise<T> => {
      return new Promise((resolve, reject) => {
        if (!isConnected) {
          reject(new Error('Not connected'));
          return;
        }

        const requestId = generateRequestId();
        setPendingOperations(prev => {
          const next = new Map(prev);
          next.set(requestId, { resolve: resolve as (value: unknown) => void, reject });
          return next;
        });

        send({ type: messageType, request_id: requestId, agent_id: agentId });
      });
    },
    [isConnected, send]
  );

  const pauseAgent = useCallback(
    (agentId: string) => performOperation<void>('pause_agent', agentId),
    [performOperation]
  );

  const resumeAgent = useCallback(
    (agentId: string) => performOperation<void>('resume_agent', agentId),
    [performOperation]
  );

  const cancelAgent = useCallback(
    (agentId: string) => performOperation<void>('cancel_agent', agentId),
    [performOperation]
  );

  const stopAgent = useCallback(
    (agentId: string) => performOperation<void>('stop_agent', agentId),
    [performOperation]
  );

  const getAgentStatus = useCallback(
    (agentId: string) => performOperation<AgentInfo>('agent_status', agentId),
    [performOperation]
  );

  // Handle incoming messages
  useEffect(() => {
    const cleanup = addMessageHandler((message: ServerMessage) => {
      switch (message.type) {
        case 'agent_list':
          if (message.request_id === pendingRequestId) {
            setAgents(message.agents);
            setIsLoading(false);
            setPendingRequestId(null);
          }
          break;

        case 'agent_spawned':
          if (spawnResolver && message.request_id === spawnResolver.requestId) {
            spawnResolver.resolve(message.agent);
            setIsSpawning(false);
            setSpawnResolver(null);
            // Add the new agent to the list
            setAgents(prev => [...prev, message.agent]);
          }
          break;

        case 'agent_status_response':
          {
            const resolver = pendingOperations.get(message.request_id);
            if (resolver) {
              resolver.resolve(message.agent);
              setPendingOperations(prev => {
                const next = new Map(prev);
                next.delete(message.request_id);
                return next;
              });
              // Update the agent in the list
              setAgents(prev =>
                prev.map(a => (a.id === message.agent.id ? message.agent : a))
              );
            }
          }
          break;

        case 'agent_ack':
          {
            const resolver = pendingOperations.get(message.request_id);
            if (resolver) {
              resolver.resolve(undefined);
              setPendingOperations(prev => {
                const next = new Map(prev);
                next.delete(message.request_id);
                return next;
              });

              // If the operation was stop, remove the agent from the list
              if (message.operation === 'stop') {
                setAgents(prev => prev.filter(a => a.id !== message.agent_id));
              } else {
                // For other operations, refresh to get updated status
                refresh();
              }
            }
          }
          break;

        case 'error':
          if (pendingRequestId) {
            setError(message.message);
            setIsLoading(false);
            setPendingRequestId(null);
          }
          if (spawnResolver) {
            spawnResolver.reject(new Error(message.message));
            setIsSpawning(false);
            setSpawnResolver(null);
          }
          // Reject any pending operations
          pendingOperations.forEach((resolver) => {
            resolver.reject(new Error(message.message));
          });
          if (pendingOperations.size > 0) {
            setPendingOperations(new Map());
          }
          break;
      }
    });

    return cleanup;
  }, [addMessageHandler, pendingRequestId, spawnResolver, pendingOperations, refresh]);

  // Auto-refresh on connect
  useEffect(() => {
    if (isConnected && autoRefresh) {
      refresh();
    }
  }, [isConnected, autoRefresh, refresh]);

  // Periodic refresh
  useEffect(() => {
    if (!isConnected || refreshInterval <= 0) {
      return;
    }

    const intervalId = setInterval(refresh, refreshInterval);
    return () => clearInterval(intervalId);
  }, [isConnected, refreshInterval, refresh]);

  return {
    agents,
    isLoading,
    isSpawning,
    error,
    refresh,
    spawnAgent,
    pauseAgent,
    resumeAgent,
    cancelAgent,
    stopAgent,
    getAgentStatus,
  };
}
