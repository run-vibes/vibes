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
    refetchInterval: 5000, // Poll every 5 seconds
  });
}
