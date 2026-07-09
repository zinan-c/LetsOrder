import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { joinGathering, listActiveGatherings } from '../api/gatherings';
import PageCard from '../components/PageCard';
import StatusPill from '../components/StatusPill';
import type { GatheringListItem } from '../types/gathering';

function formatDeadline(value: string) {
  return new Intl.DateTimeFormat(undefined, {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(value));
}

export default function JoinGatheringPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const activeGatheringsQuery = useQuery({
    queryKey: ['active-gatherings'],
    queryFn: listActiveGatherings,
    retry: false,
  });
  const joinMutation = useMutation({
    mutationFn: (gathering: GatheringListItem) => joinGathering(gathering.id, ''),
    onSuccess: async (_response, gathering) => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ['gatherings'] }),
        queryClient.invalidateQueries({ queryKey: ['participants', gathering.id] }),
      ]);
      navigate(`/menu/${gathering.invite_code}`);
    },
  });
  const activeGatherings = activeGatheringsQuery.data?.gatherings ?? [];

  return (
    <PageCard
      eyebrow="Join"
      title="Join an active gathering"
      description="This fixed entry page is designed for manual access today and can become the QR-code landing page later. Choose the gathering you want to join, then jump straight into its menu workspace."
    >
      {activeGatheringsQuery.isLoading ? (
        <p className="empty-panel-note">Loading active gatherings...</p>
      ) : null}

      {activeGatheringsQuery.isError ? (
        <p className="error">Could not load active gatherings.</p>
      ) : null}

      {!activeGatheringsQuery.isLoading && activeGatherings.length === 0 ? (
        <p className="empty-panel-note">No active gatherings are open right now.</p>
      ) : null}

      <div className="join-gathering-list">
        {activeGatherings.map((gathering) => (
          <article className="join-gathering-card" key={gathering.id}>
            <div>
              <div className="join-gathering-title-row">
                <h2>{gathering.title}</h2>
                <StatusPill>{gathering.status}</StatusPill>
              </div>
              {gathering.description ? <p>{gathering.description}</p> : null}
            </div>
            <div className="join-gathering-meta">
              <span>{gathering.item_count} dishes</span>
              <span>{gathering.participant_count} people joined</span>
              <span>Locks at {formatDeadline(gathering.expires_at)}</span>
            </div>
            <button
              disabled={joinMutation.isPending}
              type="button"
              onClick={() => joinMutation.mutate(gathering)}
            >
              {joinMutation.isPending ? 'Joining...' : 'Join gathering'}
            </button>
          </article>
        ))}
      </div>

      {joinMutation.isError ? (
        <p className="error">Could not join this gathering. Please try again.</p>
      ) : null}
    </PageCard>
  );
}
