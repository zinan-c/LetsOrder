import { useEffect } from 'react';
import type { QueryClient } from '@tanstack/react-query';
import type { User } from '../types/auth';
import { getAuthToken } from '../utils/user';
import { createWebSocketTicket } from '../api/auth';

export default function useRealtimeRefresh(
  currentUser: User | null,
  queryClient: QueryClient,
) {
  useEffect(() => {
    const token = getAuthToken();
    if (!currentUser || !token) {
      return;
    }

    let socket: WebSocket | null = null;
    let cancelled = false;
    void createWebSocketTicket().then(({ ticket }) => {
      if (cancelled) return;
      const apiBaseUrl = import.meta.env.VITE_API_BASE_URL ?? window.location.origin;
      const wsUrl = new URL('/api/ws', apiBaseUrl || window.location.origin);
      wsUrl.protocol = wsUrl.protocol === 'https:' ? 'wss:' : 'ws:';
      wsUrl.searchParams.set('ticket', ticket);
      socket = new WebSocket(wsUrl);
      socket.onmessage = handleMessage;
    }).catch(() => undefined);

    function handleMessage(event: MessageEvent) {
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
            queryKey: ['menu-ratings', message.gathering_id],
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
    }

    return () => {
      cancelled = true;
      socket?.close();
    };
  }, [currentUser, queryClient]);
}
