import { useState } from 'react';
import { Link } from 'react-router-dom';
import StatusPill from './StatusPill';
import { copyText } from '../utils/clipboard';
import { formatCompactDateTime } from '../utils/dateTime';

interface GatheringSummaryProps {
  title: string;
  description?: string | null;
  inviteCode?: string;
  expiresAt?: string;
  isLocked?: boolean;
  canManage?: boolean;
  participantCount?: number;
}

export default function GatheringSummary({
  title,
  description,
  inviteCode,
  expiresAt,
  isLocked = false,
  canManage = false,
  participantCount,
}: GatheringSummaryProps) {
  const [copyFeedback, setCopyFeedback] = useState(false);
  const inviteUrl = inviteCode
    ? `${window.location.origin}/menu/${inviteCode}`
    : null;

  function showCopyFeedback() {
    setCopyFeedback(true);
    window.setTimeout(() => setCopyFeedback(false), 1200);
  }

  async function handleCopyInvite() {
    if (!inviteUrl) {
      return;
    }

    try {
      await copyText(inviteUrl);
      showCopyFeedback();
    } catch {
      return;
    }
  }

  return (
    <aside className="summary-card">
      <div className="summary-topline">
        <StatusPill>Current menu</StatusPill>
        {participantCount ? <span>{participantCount} people joined</span> : null}
      </div>
      <h2>{title}</h2>
      {description ? <p>{description}</p> : null}
      <div className="summary-lines">
        <p>Menu will lock at {formatCompactDateTime(expiresAt)}.</p>
        <p>
          Invite URL: {inviteUrl ? <span>{inviteUrl}</span> : 'Not ready'}
          {inviteUrl ? (
            <span className="button-feedback-wrap">
              <button
                className="ghost-button mini-copy-button"
                type="button"
                onClick={handleCopyInvite}
              >
                Copy
              </button>
              {copyFeedback ? <span className="button-feedback">Copied</span> : null}
            </span>
          ) : null}
        </p>
      </div>
      {inviteCode ? (
        <div className="summary-actions">
          {canManage ? (
            <Link className="button-link secondary" to={`/host/${inviteCode}`}>
              Host controls
            </Link>
          ) : null}
          {isLocked ? (
            <Link className="button-link secondary" to={`/review/${inviteCode}`}>
              Review
            </Link>
          ) : null}
        </div>
      ) : null}
    </aside>
  );
}
