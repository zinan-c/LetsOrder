import { FormEvent, useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { Link, useNavigate } from 'react-router-dom';
import { createGathering } from '../api/gatherings';
import PageCard from '../components/PageCard';
import { copyText } from '../utils/clipboard';
import { toDateTimeLocalValue } from '../utils/dateTime';

function defaultExpiry() {
  const date = new Date();
  date.setDate(date.getDate() + 1);
  return toDateTimeLocalValue(date.toISOString());
}

const systemAdminHostNameMessage =
  '这是系统管理员账号名称，请使用系统管理员账号及密码登陆';

export default function CreateGatheringPage() {
  const navigate = useNavigate();
  const [title, setTitle] = useState('');
  const [hostName, setHostName] = useState('');
  const [description, setDescription] = useState('');
  const [expiresAt, setExpiresAt] = useState(defaultExpiry);
  const [copyFeedback, setCopyFeedback] = useState(false);
  const isSystemAdminHostName = hostName.trim() === 'suite-admin';

  const mutation = useMutation({
    mutationFn: createGathering,
    onSuccess: (response) => {
      localStorage.setItem(
        `letsorder:${response.gathering.invite_code}:participant_id`,
        response.host.id,
      );
      localStorage.setItem(
        `letsorder:${response.gathering.invite_code}:access_token`,
        response.access_token,
      );
      navigate(`/host/${response.gathering.invite_code}`);
    },
  });

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (isSystemAdminHostName) {
      return;
    }

    mutation.mutate({
      title,
      description,
      host_name: hostName,
      expires_at: new Date(expiresAt).toISOString(),
    });
  }

  const inviteUrl = mutation.data
    ? `${window.location.origin}/menu/${mutation.data.gathering.invite_code}`
    : null;

  async function handleCopyInvite() {
    if (!inviteUrl) {
      return;
    }

    try {
      await copyText(inviteUrl);
      setCopyFeedback(true);
      window.setTimeout(() => setCopyFeedback(false), 1200);
    } catch {
      return;
    }
  }

  return (
    <div>
      <PageCard
        eyebrow="GATHERING MENU"
        title="Create a GATHERING"
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
            {isSystemAdminHostName ? (
              <span className="field-note error-note">
                {systemAdminHostNameMessage}
              </span>
            ) : null}
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
            <textarea
              value={description}
              onChange={(event) => setDescription(event.target.value)}
              placeholder="Tell people what kind of meal this is."
            />
          </label>

          <button disabled={mutation.isPending || isSystemAdminHostName} type="submit">
            {mutation.isPending ? 'Creating invitation...' : 'Create invitation'}
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
              <span className="button-feedback-wrap">
                <button type="button" onClick={handleCopyInvite}>
                  Copy link
                </button>
                {copyFeedback ? <span className="button-feedback">Copied</span> : null}
              </span>
              <a className="button-link secondary" href={inviteUrl}>
                Open menu
              </a>
              <Link className="button-link secondary" to="/menus">
                View menus
              </Link>
            </div>
          </div>
        ) : null}
      </PageCard>
    </div>
  );
}
