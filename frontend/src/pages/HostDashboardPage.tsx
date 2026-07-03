import { useState, type ReactNode } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useParams } from 'react-router-dom';
import {
  getGatheringByInviteCode,
  listActivityLogs,
  listParticipants,
  lockGathering,
} from '../api/gatherings';
import PageCard from '../components/PageCard';
import StatusPill from '../components/StatusPill';
import type { ActivityLog, Participant } from '../types/gathering';

function formatDateTime(value?: string | null) {
  if (!value) {
    return 'No activity yet';
  }

  return new Intl.DateTimeFormat(undefined, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(value));
}

function formatAction(log: ActivityLog) {
  return log.action.replaceAll('_', ' ');
}

export default function HostDashboardPage() {
  const { inviteCode } = useParams();
  const queryClient = useQueryClient();
  const [showAllParticipants, setShowAllParticipants] = useState(false);
  const [showAllActivity, setShowAllActivity] = useState(false);
  const inviteUrl = `${window.location.origin}/menu/${inviteCode}`;

  const gatheringQuery = useQuery({
    queryKey: ['gathering', inviteCode],
    queryFn: () => getGatheringByInviteCode(inviteCode ?? ''),
    enabled: Boolean(inviteCode),
    retry: false,
  });
  const gathering = gatheringQuery.data?.gathering;

  const participantsQuery = useQuery({
    queryKey: ['participants', gathering?.id],
    queryFn: () => listParticipants(gathering?.id ?? ''),
    enabled: Boolean(gathering?.id),
    retry: false,
  });

  const activityQuery = useQuery({
    queryKey: ['activity-logs', gathering?.id],
    queryFn: () => listActivityLogs(gathering?.id ?? ''),
    enabled: Boolean(gathering?.id),
    retry: false,
  });

  const lockMutation = useMutation({
    mutationFn: () => lockGathering(gathering?.id ?? ''),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['gathering', inviteCode] });
      await queryClient.invalidateQueries({ queryKey: ['gatherings'] });
      await queryClient.invalidateQueries({ queryKey: ['activity-logs', gathering?.id] });
    },
  });

  const participants = participantsQuery.data?.participants ?? [];
  const recentParticipants = participants.slice(0, 4);
  const activityLogs = activityQuery.data?.activity_logs ?? [];
  const recentActivityLogs = activityLogs.slice(0, 4);

  return (
    <div className="dashboard-grid">
      <PageCard
        eyebrow="Host controls"
        title="Gathering on track"
        titleClassName="nowrap-title"
        description="Copy the invite link, adjust the editing window, and lock the menu when everyone has had their say."
      >
        <div className="result-panel">
          <p>Invitation URL</p>
          <a href={inviteUrl}>{inviteUrl}</a>
          <div className="action-row">
            <button type="button">Copy invite</button>
          </div>
        </div>

        <div className="control-grid">
          <label>
            Menu editing deadline
            <input
              type="datetime-local"
              defaultValue={
                gathering?.expires_at
                  ? new Date(gathering.expires_at).toISOString().slice(0, 16)
                  : ''
              }
            />
          </label>
          <button type="button">Update deadline</button>
          <button
            className="danger-button"
            disabled={!gathering || gathering.is_locked || lockMutation.isPending}
            type="button"
            onClick={() => lockMutation.mutate()}
          >
            {gathering?.is_locked
              ? 'Menu locked'
              : lockMutation.isPending
                ? 'Locking...'
                : 'Lock menu now'}
          </button>
        </div>
        {lockMutation.isError ? (
          <p className="error">Could not lock this menu.</p>
        ) : null}
      </PageCard>

      <section className="dashboard-panel">
        <div className="panel-header">
          <div>
            <p className="card-kicker">Participants</p>
            <h2>{participants.length} joined</h2>
          </div>
          <StatusPill tone={gathering?.is_locked ? 'neutral' : 'warm'}>
            {gathering?.is_locked ? 'Locked' : 'Active'}
          </StatusPill>
        </div>
        <ParticipantList participants={recentParticipants} />
        {participants.length > 4 ? (
          <button
            className="ghost-button panel-more-button"
            type="button"
            onClick={() => setShowAllParticipants(true)}
          >
            More...
          </button>
        ) : null}
      </section>

      <section className="dashboard-panel">
        <div className="panel-header">
          <div>
            <p className="card-kicker">Activity log</p>
            <h2>Recent changes</h2>
          </div>
        </div>
        <ActivityList logs={recentActivityLogs} />
        {activityLogs.length > 4 ? (
          <button
            className="ghost-button panel-more-button"
            type="button"
            onClick={() => setShowAllActivity(true)}
          >
            More...
          </button>
        ) : null}
      </section>

      {showAllParticipants ? (
        <Modal title="All participants" onClose={() => setShowAllParticipants(false)}>
          <ParticipantList participants={participants} />
        </Modal>
      ) : null}

      {showAllActivity ? (
        <Modal title="All activity" onClose={() => setShowAllActivity(false)}>
          <ActivityList logs={activityLogs} />
        </Modal>
      ) : null}
    </div>
  );
}

function ParticipantList({ participants }: { participants: Participant[] }) {
  if (participants.length === 0) {
    return <p className="empty-panel-note">No participants yet.</p>;
  }

  return (
    <div className="participant-list">
      {participants.map((participant) => (
        <div className="participant-row" key={participant.id}>
          <div>
            <strong>{participant.display_name}</strong>
            <span>{participant.role}</span>
          </div>
          <time>
            {formatDateTime(
              participant.last_menu_activity_at ?? participant.joined_at,
            )}
          </time>
        </div>
      ))}
    </div>
  );
}

function ActivityList({ logs }: { logs: ActivityLog[] }) {
  if (logs.length === 0) {
    return <p className="empty-panel-note">No activity yet.</p>;
  }

  return (
    <ol className="activity-list">
      {logs.map((log) => (
        <li key={log.id}>
          <span>{log.actor_name ?? 'System'}</span> {formatAction(log)}
          <time>{formatDateTime(log.created_at)}</time>
        </li>
      ))}
    </ol>
  );
}

function Modal({
  title,
  children,
  onClose,
}: {
  title: string;
  children: ReactNode;
  onClose: () => void;
}) {
  return (
    <div className="modal-overlay" role="presentation" onClick={onClose}>
      <section
        aria-modal="true"
        className="confirm-modal large-modal"
        role="dialog"
        onClick={(event) => event.stopPropagation()}
      >
        <div className="panel-header">
          <h2>{title}</h2>
          <button className="icon-button" type="button" onClick={onClose}>
            ×
          </button>
        </div>
        {children}
      </section>
    </div>
  );
}
