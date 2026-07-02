import { mockGathering } from '../data/mockGathering';
import StatusPill from './StatusPill';

export default function GatheringSummary() {
  return (
    <aside className="summary-card">
      <div className="summary-topline">
        <StatusPill>Active menu</StatusPill>
        <span>{mockGathering.participantCount} people joined</span>
      </div>
      <h2>{mockGathering.title}</h2>
      <p>{mockGathering.description}</p>
      <dl className="summary-list">
        <div>
          <dt>Host</dt>
          <dd>{mockGathering.hostName}</dd>
        </div>
        <div>
          <dt>Menu locks</dt>
          <dd>Jul 3, 2026 · 6:00 PM</dd>
        </div>
        <div>
          <dt>Invite</dt>
          <dd>/g/{mockGathering.inviteCode}</dd>
        </div>
      </dl>
    </aside>
  );
}
