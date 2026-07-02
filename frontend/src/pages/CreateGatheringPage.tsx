import { FormEvent, useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { createGathering } from '../api/gatherings';
import GatheringSummary from '../components/GatheringSummary';
import PageCard from '../components/PageCard';

function defaultExpiry() {
  const date = new Date();
  date.setDate(date.getDate() + 1);
  return date.toISOString().slice(0, 16);
}

export default function CreateGatheringPage() {
  const [title, setTitle] = useState('');
  const [hostName, setHostName] = useState('');
  const [expiresAt, setExpiresAt] = useState(defaultExpiry);

  const mutation = useMutation({
    mutationFn: createGathering,
  });

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    mutation.mutate({
      title,
      host_name: hostName,
      expires_at: new Date(expiresAt).toISOString(),
    });
  }

  const inviteUrl = mutation.data
    ? `${window.location.origin}/g/${mutation.data.gathering.invite_code}`
    : null;

  return (
    <div className="two-column">
      <PageCard
        eyebrow="Family gathering menu"
        title="Create a shared menu space"
        description="Start a gathering, invite family members, and let everyone help shape the menu before it locks."
      >
        <form className="form-grid" onSubmit={handleSubmit}>
          <label>
            Gathering title
            <input
              required
              value={title}
              onChange={(event) => setTitle(event.target.value)}
              placeholder="Saturday hotpot night"
            />
          </label>

          <label>
            Host name
            <input
              required
              value={hostName}
              onChange={(event) => setHostName(event.target.value)}
              placeholder="Aunt May"
            />
          </label>

          <label>
            Menu editing expires at
            <input
              required
              type="datetime-local"
              value={expiresAt}
              onChange={(event) => setExpiresAt(event.target.value)}
            />
          </label>

          <label>
            Description
            <textarea placeholder="Tell people what kind of meal this is." />
          </label>

          <button disabled={mutation.isPending} type="submit">
            {mutation.isPending ? 'Creating...' : 'Create invitation'}
          </button>
        </form>

        {mutation.isError ? (
          <p className="error">
            Could not create gathering. Is the backend running?
          </p>
        ) : null}

        {inviteUrl ? (
          <div className="result-panel">
            <p>Your invitation link is ready:</p>
            <a href={inviteUrl}>{inviteUrl}</a>
            <div className="action-row">
              <button type="button">Copy link</button>
              <a className="button-link secondary" href={inviteUrl}>
                Preview invite
              </a>
            </div>
          </div>
        ) : null}
      </PageCard>
      <GatheringSummary />
    </div>
  );
}
