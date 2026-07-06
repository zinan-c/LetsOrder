import { useEffect, useState, type ReactNode } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Link, useParams } from 'react-router-dom';
import {
  getGatheringByInviteCode,
  listActivityLogs,
  listParticipants,
  lockGathering,
  updateGatheringDeadline,
} from '../api/gatherings';
import PageCard from '../components/PageCard';
import StatusPill from '../components/StatusPill';
import type { ActivityLog, Participant } from '../types/gathering';
import { copyText } from '../utils/clipboard';
import { formatDateTime, toDateTimeLocalValue } from '../utils/dateTime';
import { getCookieUser, getCurrentUser, USER_CHANGED_EVENT } from '../utils/user';

type ActivityDetail = {
  field?: string;
  before?: unknown;
  after?: unknown;
};

function parseDetail(log: ActivityLog) {
  if (!log.detail) {
    return null;
  }

  try {
    return JSON.parse(log.detail) as ActivityDetail;
  } catch {
    return null;
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function formatValue(value: unknown) {
  if (value === null || value === undefined || value === '') {
    return 'empty';
  }

  if (typeof value === 'string' && value.includes('T')) {
    const date = new Date(value);
    if (!Number.isNaN(date.getTime())) {
      return formatDateTime(value);
    }
  }

  return String(value);
}

function formatChanges(log: ActivityLog) {
  const detail = parseDetail(log);
  if (!detail || detail.before === undefined || detail.after === undefined) {
    return '';
  }

  if (
    (log.action === 'gathering_deadline_updated' ||
      log.action === 'menu_reopened') &&
    isRecord(detail.before) &&
    isRecord(detail.after)
  ) {
    return `from ${formatValue(detail.before.expires_at)} to ${formatValue(
      detail.after.expires_at,
    )}`;
  }

  if (detail.field) {
    return `${detail.field.replaceAll('_', ' ')}: ${formatValue(
      detail.before,
    )} → ${formatValue(detail.after)}`;
  }

  if (!isRecord(detail.before) || !isRecord(detail.after)) {
    return '';
  }

  const beforeRecord = detail.before;
  const afterRecord = detail.after;
  const changes = Object.keys(afterRecord)
    .filter((key) => {
      const beforeValue = beforeRecord[key];
      const afterValue = afterRecord[key];
      return JSON.stringify(beforeValue) !== JSON.stringify(afterValue);
    })
    .map(
      (key) =>
        `${key.replaceAll('_', ' ')}: ${formatValue(
          beforeRecord[key],
        )} → ${formatValue(afterRecord[key])}`,
    );

  return changes.join('; ');
}

function formatFieldChange(log: ActivityLog) {
  const detail = parseDetail(log);
  if (!detail || detail.before === undefined || detail.after === undefined) {
    return '';
  }

  const before = formatValue(detail.before);
  const after = formatValue(detail.after);

  switch (log.action) {
    case 'menu_item_name_changed':
      return `renamed a menu item from ${before} to ${after}`;
    case 'menu_item_category_changed':
      return `moved a menu item from ${before} to ${after}`;
    case 'menu_item_quantity_changed':
      return `changed the item quantity from ${before} to ${after}`;
    case 'menu_item_unit_changed':
      return `changed the item unit from ${before} to ${after}`;
    case 'menu_item_owner_changed':
      return `changed the Chef from ${before} to ${after}`;
    case 'menu_item_reference_url_changed':
      return `updated the reference link from ${before} to ${after}`;
    case 'menu_item_note_changed':
      return `updated the item note from ${before} to ${after}`;
    case 'menu_item_status_changed':
      return `changed the item status from ${before} to ${after}`;
    case 'menu_item_cancelled':
      return `cancelled a menu item`;
    case 'photo_caption_updated':
      return `renamed a photo from ${before} to ${after}`;
    default:
      return '';
  }
}

function formatAction(log: ActivityLog) {
  const fieldChange = formatFieldChange(log);
  if (fieldChange) {
    return fieldChange;
  }

  if (log.action === 'menu_item_created') {
    return 'added a menu item';
  }

  if (log.action === 'participant_joined') {
    return 'joined the gathering';
  }

  if (log.action === 'gathering_locked') {
    return 'locked the menu';
  }

  if (log.action === 'gathering_auto_locked') {
    return 'auto-locked the menu after the deadline';
  }

  if (log.action === 'gathering_archived') {
    return 'archived the gathering';
  }

  if (log.action === 'photo_uploaded') {
    return 'uploaded a photo';
  }

  if (log.action === 'photo_deleted') {
    return 'deleted a photo';
  }

  if (log.action === 'gathering_deadline_updated') {
    const changes = formatChanges(log);
    return changes ? `updated the menu deadline ${changes}` : 'updated the menu deadline';
  }

  if (log.action === 'menu_reopened') {
    const changes = formatChanges(log);
    return changes ? `reopened the menu ${changes}` : 'reopened the menu';
  }

  const action = log.action.replaceAll('_', ' ');
  const changes = formatChanges(log);
  return changes ? `${action} (${changes})` : action;
}

export default function HostDashboardPage() {
  const { inviteCode } = useParams();
  const queryClient = useQueryClient();
  const [showAllParticipants, setShowAllParticipants] = useState(false);
  const [showAllActivity, setShowAllActivity] = useState(false);
  const [deadline, setDeadline] = useState('');
  const [currentUser, setCurrentUser] = useState(() => getCookieUser());
  const [buttonFeedback, setButtonFeedback] = useState<
    'copy' | 'deadline' | 'lock' | null
  >(null);
  const inviteUrl = `${window.location.origin}/menu/${inviteCode}`;
  const isAdmin = getCurrentUser()?.role === 'admin';

  function showButtonFeedback(feedback: 'copy' | 'deadline' | 'lock') {
    setButtonFeedback(feedback);
    window.setTimeout(() => setButtonFeedback(null), 1200);
  }

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
    onMutate: () => {
      setButtonFeedback(null);
    },
    onSuccess: async (response) => {
      setDeadline(toDateTimeLocalValue(response.gathering.expires_at));
      showButtonFeedback('lock');
      queryClient.setQueryData(['gathering', inviteCode], response);
      await queryClient.invalidateQueries({ queryKey: ['gathering', inviteCode] });
      await queryClient.invalidateQueries({ queryKey: ['gatherings'] });
      await queryClient.invalidateQueries({ queryKey: ['activity-logs', gathering?.id] });
    },
  });

  const updateDeadlineMutation = useMutation({
    mutationFn: () =>
      updateGatheringDeadline(
        gathering?.id ?? '',
        new Date(deadline).toISOString(),
      ),
    onMutate: () => {
      setButtonFeedback(null);
    },
    onSuccess: async (response) => {
      setDeadline(toDateTimeLocalValue(response.gathering.expires_at));
      showButtonFeedback('deadline');
      queryClient.setQueryData(['gathering', inviteCode], response);
      await queryClient.invalidateQueries({ queryKey: ['gathering', inviteCode] });
      await queryClient.invalidateQueries({ queryKey: ['gatherings'] });
      await queryClient.invalidateQueries({ queryKey: ['activity-logs', gathering?.id] });
    },
  });

  useEffect(() => {
    setDeadline(toDateTimeLocalValue(gathering?.expires_at));
  }, [gathering?.expires_at]);

  useEffect(() => {
    function handleUserChanged(event: Event) {
      const user = event instanceof CustomEvent ? event.detail : null;
      setCurrentUser(user?.display_name ?? '');
    }

    window.addEventListener(USER_CHANGED_EVENT, handleUserChanged);

    return () => {
      window.removeEventListener(USER_CHANGED_EVENT, handleUserChanged);
    };
  }, []);

  const participants = participantsQuery.data?.participants ?? [];
  const recentParticipants = participants.slice(0, 4);
  const activityLogs = activityQuery.data?.activity_logs ?? [];
  const recentActivityLogs = activityLogs.slice(0, 4);

  async function handleCopyInvite() {
    try {
      await copyText(inviteUrl);
      showButtonFeedback('copy');
    } catch {
      return;
    }
  }

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
            <span className="button-feedback-wrap">
              <button type="button" onClick={handleCopyInvite}>
                Copy invite
              </button>
              {buttonFeedback === 'copy' ? (
                <span className="button-feedback">Copied</span>
              ) : null}
            </span>
            <Link className="button-link secondary" to={`/menu/${inviteCode}`}>
              Enter menu
            </Link>
          </div>
        </div>

        {isAdmin ? (
          <>
            <div className="control-grid">
              <label>
                Menu editing deadline
                <input
                  type="datetime-local"
                  value={deadline}
                  disabled={!gathering}
                  onChange={(event) => setDeadline(event.target.value)}
                />
              </label>
              <span className="button-feedback-wrap">
                <button
                  disabled={
                    !gathering ||
                    !deadline ||
                    updateDeadlineMutation.isPending
                  }
                  type="button"
                  onClick={() => updateDeadlineMutation.mutate()}
                >
                  {updateDeadlineMutation.isPending
                    ? 'Updating...'
                    : 'Update deadline'}
                </button>
                {buttonFeedback === 'deadline' ? (
                  <span className="button-feedback">Updated</span>
                ) : null}
              </span>
              <span className="button-feedback-wrap">
                <button
                  className="danger-button"
                  disabled={
                    !gathering || gathering.is_locked || lockMutation.isPending
                  }
                  type="button"
                  onClick={() => lockMutation.mutate()}
                >
                  {gathering?.is_locked
                    ? 'Already locked'
                    : lockMutation.isPending
                      ? 'Locking...'
                      : 'Lock menu now'}
                </button>
                {buttonFeedback === 'lock' ? (
                  <span className="button-feedback">Already locked</span>
                ) : null}
              </span>
            </div>
            {lockMutation.isError && !gathering?.is_locked ? (
              <p className="error">Could not lock this menu.</p>
            ) : null}
            {updateDeadlineMutation.isError ? (
              <p className="error">Could not update the deadline.</p>
            ) : null}
          </>
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
