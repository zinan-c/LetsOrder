import { Link, useParams } from 'react-router-dom';
import PageCard from '../components/PageCard';
import StatusPill from '../components/StatusPill';
import {
  mockActivityLogs,
  mockGathering,
  mockParticipants,
} from '../data/mockGathering';

export default function HostDashboardPage() {
  const { inviteCode } = useParams();
  const inviteUrl = `${window.location.origin}/menu/${inviteCode}`;

  return (
    <div className="dashboard-grid">
      <PageCard
        eyebrow="Host controls"
        title="Keep the gathering on track"
        description="Copy the invite link, adjust the editing window, and lock the menu when everyone has had their say."
      >
        <div className="result-panel">
          <p>Invitation URL</p>
          <a href={inviteUrl}>{inviteUrl}</a>
          <div className="action-row">
            <button type="button">Copy invite</button>
            <Link className="button-link secondary" to={`/menu/${inviteCode}`}>
              Preview invite
            </Link>
          </div>
        </div>

        <div className="control-grid">
          <label>
            Menu editing deadline
            <input type="datetime-local" defaultValue="2026-07-03T18:00" />
          </label>
          <button type="button">Update deadline</button>
          <button className="danger-button" type="button">
            Lock menu now
          </button>
        </div>
      </PageCard>

      <section className="dashboard-panel">
        <div className="panel-header">
          <div>
            <p className="card-kicker">Participants</p>
            <h2>{mockGathering.participantCount} joined</h2>
          </div>
          <StatusPill>Active</StatusPill>
        </div>
        <div className="participant-list">
          {mockParticipants.map((participant) => (
            <div className="participant-row" key={participant.id}>
              <div>
                <strong>{participant.name}</strong>
                <span>{participant.role}</span>
              </div>
              <time>{participant.joinedAt}</time>
            </div>
          ))}
        </div>
      </section>

      <section className="dashboard-panel">
        <div className="panel-header">
          <div>
            <p className="card-kicker">Activity log</p>
            <h2>Recent changes</h2>
          </div>
        </div>
        <ol className="activity-list">
          {mockActivityLogs.map((log) => (
            <li key={log}>{log}</li>
          ))}
        </ol>
      </section>
    </div>
  );
}
