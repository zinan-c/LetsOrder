import { useEffect } from 'react';
import type { QueryClient } from '@tanstack/react-query';
import type { User } from '../types/auth';
import { getAuthToken } from '../utils/user';

export default function useRealtimeRefresh(
  currentUser: User | null,
  queryClient: QueryClient,
) {
  useEffect(() => {
    const token = getAuthToken();
    if (!currentUser || !token) {
      return;
    }

    const apiBaseUrl = import.meta.env.VITE_API_BASE_URL ?? window.location.origin;
    const wsUrl = new URL('/api/ws', apiBaseUrl || window.location.origin);
    wsUrl.protocol = wsUrl.protocol === 'https:' ? 'wss:' : 'ws:';
    wsUrl.searchParams.set('token', token);

    const socket = new WebSocket(wsUrl);
    socket.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data) as {
          event?: string;
          gathering_id?: string | null;
        };

        if (message.event !== 'refresh') {
          return;
        }

        queryClient.invalidateQueries({ queryKey: ['gatherings'] });
        queryClient.invalidateQueries({ queryKey: ['gathering'] });

        if (message.gathering_id) {
          queryClient.invalidateQueries({
            queryKey: ['menu-items', message.gathering_id],
          });
          queryClient.invalidateQueries({
            queryKey: ['participants', message.gathering_id],
          });
          queryClient.invalidateQueries({
            queryKey: ['activity-logs', message.gathering_id],
          });
          queryClient.invalidateQueries({
            queryKey: ['photos', message.gathering_id],
          });
        }
      } catch {
        return;
      }
    };

    return () => {
      socket.close();
    };
  }, [currentUser, queryClient]);
}
