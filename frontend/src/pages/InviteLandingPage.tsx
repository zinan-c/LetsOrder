import { FormEvent, useState } from 'react';
import { useMutation, useQuery } from '@tanstack/react-query';
import { Link, useParams } from 'react-router-dom';
import { getGatheringByInviteCode, joinGathering } from '../api/gatherings';
import GatheringSummary from '../components/GatheringSummary';
import PageCard from '../components/PageCard';
import { mockMenuItems } from '../data/mockGathering';

export default function InviteLandingPage() {
  const { inviteCode } = useParams();
  const [displayName, setDisplayName] = useState('');
  const gatheringQuery = useQuery({
    queryKey: ['gathering', inviteCode],
    queryFn: () => getGatheringByInviteCode(inviteCode ?? ''),
    enabled: Boolean(inviteCode),
    retry: false,
  });
  const joinMutation = useMutation({
    mutationFn: async () => {
      if (!gatheringQuery.data || !inviteCode) {
        throw new Error('Gathering is not loaded yet.');
      }

      return joinGathering(gatheringQuery.data.gathering.id, displayName);
    },
    onSuccess: (response) => {
      if (!inviteCode) {
        return;
      }

      localStorage.setItem(
        `letsorder:${inviteCode}:participant_id`,
        response.participant.id,
      );
      localStorage.setItem(
        `letsorder:${inviteCode}:access_token`,
        response.access_token,
      );
    },
  });

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    joinMutation.mutate();
  }

  const canEnterMenu =
    joinMutation.isSuccess ||
    gatheringQuery.isError ||
    Boolean(
      inviteCode && localStorage.getItem(`letsorder:${inviteCode}:participant_id`),
    );

  return (
    <div className="two-column">
      <PageCard
        eyebrow="You are invited"
        title="Join this family menu board"
        description="Add your name first. Then you can help add dishes, update prep notes, and see what everyone is bringing."
      >
        <form className="form-grid compact" onSubmit={handleSubmit}>
          <label>
            Your display name
            <input
              required
              value={displayName}
              onChange={(event) => setDisplayName(event.target.value)}
              placeholder="Grandma Lin"
            />
          </label>
          {canEnterMenu ? (
            <Link className="button-link" to={`/g/${inviteCode}/menu`}>
              {gatheringQuery.isError ? 'Enter prototype menu' : 'Enter menu'}
            </Link>
          ) : (
            <button disabled={joinMutation.isPending} type="submit">
              {joinMutation.isPending ? 'Joining...' : 'Join menu'}
            </button>
          )}
        </form>

        {joinMutation.isError ? (
          <p className="error">
            Could not join through the API. Is the backend running?
          </p>
        ) : null}

        <div className="mini-menu-preview">
          <p className="card-kicker">Current menu preview</p>
          {mockMenuItems.slice(0, 3).map((item) => (
            <div className="mini-row" key={item.id}>
              <span>{item.name}</span>
              <strong>
                {item.quantity} {item.unit}
              </strong>
            </div>
          ))}
        </div>
      </PageCard>
      <GatheringSummary />
    </div>
  );
}
