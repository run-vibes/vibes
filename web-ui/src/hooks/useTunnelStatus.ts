import { useQuery } from '@tanstack/react-query';

interface TunnelStatus {
  state: 'disabled' | 'starting' | 'connected' | 'reconnecting' | 'failed' | 'stopped';
  mode: 'quick' | 'named' | null;
  url: string | null;
  tunnel_name: string | null;
  error: string | null;
}

export function useTunnelStatus() {
  return useQuery<TunnelStatus>({
    queryKey: ['tunnel-status'],
    queryFn: async () => {
      const response = await fetch('/api/tunnel/status');
      if (!response.ok) {
        throw new Error('Failed to fetch tunnel status');
      }
      return response.json();
    },
    // Adaptive polling: slower for stable states, faster for transitioning states
    refetchInterval: (query) => {
      const state = query.state.data?.state;
      if (state === 'connected' || state === 'disabled' || state === 'stopped') {
        return 30000; // Poll every 30 seconds in stable states
      }
      return 5000; // Poll every 5 seconds during transitions
    },
  });
}
