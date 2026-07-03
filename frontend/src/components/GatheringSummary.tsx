import { Link } from 'react-router-dom';
import StatusPill from './StatusPill';

interface GatheringSummaryProps {
  title: string;
  description?: string | null;
  inviteCode?: string;
  expiresAt?: string;
  participantCount?: number;
}

function formatDateTime(value?: string) {
  if (!value) {
    return 'Not set';
  }

  return new Intl.DateTimeFormat(undefined, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(value));
}

export default function GatheringSummary({
  title,
  description,
  inviteCode,
  expiresAt,
  participantCount,
}: GatheringSummaryProps) {
  return (
    <aside className="summary-card">
      <div className="summary-topline">
        <StatusPill>Current menu</StatusPill>
        {participantCount ? <span>{participantCount} people joined</span> : null}
      </div>
      <h2>{title}</h2>
      {description ? <p>{description}</p> : null}
      <dl className="summary-list">
        <div>
          <dt>Menu locks</dt>
          <dd>{formatDateTime(expiresAt)}</dd>
        </div>
        <div>
          <dt>Invite</dt>
          <dd>{inviteCode ? `/menu/${inviteCode}` : 'Not ready'}</dd>
        </div>
      </dl>
      {inviteCode ? (
        <div className="summary-actions">
          <Link className="button-link secondary" to={`/host/${inviteCode}`}>
            Host controls
          </Link>
          <Link className="button-link secondary" to={`/review/${inviteCode}`}>
            Review
          </Link>
        </div>
      ) : null}
    </aside>
  );
}
