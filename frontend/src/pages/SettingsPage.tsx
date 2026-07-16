import { FormEvent, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { listMembers, logout, updateAccount, updateMember } from '../api/auth';
import PageCard from '../components/PageCard';
import type { User } from '../types/auth';
import { clearAuthSession, getCurrentUser, updateStoredUser } from '../utils/user';

type SettingsTab = 'account' | 'members';

interface MemberEditorProps {
  member: User;
}

function MemberEditor({ member }: MemberEditorProps) {
  const queryClient = useQueryClient();
  const isSystemAdmin = member.username === 'suite-admin';
  const [displayName, setDisplayName] = useState(member.display_name);
  const [password, setPassword] = useState('');
  const [saved, setSaved] = useState(false);
  const mutation = useMutation({
    mutationFn: () =>
      updateMember(member.id, {
        display_name: displayName,
        password: password || undefined,
      }),
    onSuccess: async () => {
      setPassword('');
      setSaved(true);
      await queryClient.invalidateQueries({ queryKey: ['members'] });
      window.setTimeout(() => setSaved(false), 1200);
    },
  });

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    mutation.mutate();
  }

  return (
    <form className="member-card" onSubmit={handleSubmit}>
      <div className="member-card-heading">
        <div>
          <h2>{member.display_name}</h2>
          <p>{member.username}</p>
        </div>
        <span>{member.role}</span>
      </div>
      <div className="member-card-fields">
        <label>
          Display name
          <input
            required
            disabled={isSystemAdmin}
            value={displayName}
            onChange={(event) => setDisplayName(event.target.value)}
          />
        </label>
        <label>
          New password
          <input
            disabled={isSystemAdmin}
            type="password"
            value={password}
            placeholder="Leave blank to keep current password"
            onChange={(event) => setPassword(event.target.value)}
          />
        </label>
      </div>
      {isSystemAdmin ? (
        <p className="empty-panel-note">
          System admin credentials are managed by server configuration.
        </p>
      ) : null}
      {mutation.isError ? <p className="error">Could not update this member.</p> : null}
      {saved ? <p className="empty-panel-note">Member updated.</p> : null}
      <button disabled={mutation.isPending || isSystemAdmin} type="submit">
        {mutation.isPending ? 'Saving...' : 'Save member'}
      </button>
    </form>
  );
}

export default function SettingsPage() {
  const currentUser = getCurrentUser();
  const isSystemAdmin = currentUser?.username === 'suite-admin';
  const isAdmin = currentUser?.role === 'admin';
  const [activeTab, setActiveTab] = useState<SettingsTab>('account');
  const [displayName, setDisplayName] = useState(currentUser?.display_name ?? '');
  const [password, setPassword] = useState('');
  const [saved, setSaved] = useState(false);
  const mutation = useMutation({
    mutationFn: () =>
      updateAccount({
        display_name: displayName,
        password: password || undefined,
      }),
    onSuccess: (response) => {
      updateStoredUser(response.user);
      setPassword('');
      setSaved(true);
      window.setTimeout(() => setSaved(false), 1200);
    },
  });
  const membersQuery = useQuery({
    queryKey: ['members'],
    queryFn: listMembers,
    enabled: Boolean(isAdmin && activeTab === 'members'),
    retry: false,
  });
  const logoutMutation = useMutation({
    mutationFn: logout,
    onSettled: () => {
      clearAuthSession();
    },
  });

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    mutation.mutate();
  }

  if (!currentUser) {
    return (
      <PageCard
        eyebrow="Setting"
        title="Please log in"
        description="Account settings are available after login."
      />
    );
  }

  return (
    <div className="settings-layout">
      <aside className="settings-sidebar">
        <p className="card-kicker">Setting</p>
        <button
          className={`settings-nav-item ${activeTab === 'account' ? 'active' : ''}`}
          type="button"
          onClick={() => setActiveTab('account')}
        >
          Account Setting
        </button>
        {isAdmin ? (
          <button
            className={`settings-nav-item ${activeTab === 'members' ? 'active' : ''}`}
            type="button"
            onClick={() => setActiveTab('members')}
          >
            Member Management
          </button>
        ) : null}
      </aside>

      {activeTab === 'account' ? (
        <PageCard
          eyebrow="Account Setting"
          title="Manage your account"
          description="Update your display name or set a new password."
        >
          <form className="form-grid" onSubmit={handleSubmit}>
            <label>
              Username
              <input disabled value={currentUser.username} />
            </label>
            <label>
              Display name
              <input
                required
                disabled={isSystemAdmin}
                value={displayName}
                onChange={(event) => setDisplayName(event.target.value)}
              />
            </label>
            <label>
              New password
              <input
                disabled={isSystemAdmin}
                type="password"
                value={password}
                placeholder="Leave blank to keep current password"
                onChange={(event) => setPassword(event.target.value)}
              />
            </label>
            {isSystemAdmin ? (
              <p className="empty-panel-note">
                System admin credentials are managed by server configuration.
              </p>
            ) : null}
            {mutation.isError ? (
              <p className="error">Could not update account settings.</p>
            ) : null}
            {saved ? <p className="empty-panel-note">Account updated.</p> : null}
            <div className="action-row">
              <button disabled={mutation.isPending} type="submit">
                {mutation.isPending ? 'Saving...' : 'Save changes'}
              </button>
              <button
                className="ghost-button"
                disabled={logoutMutation.isPending}
                type="button"
                onClick={() => logoutMutation.mutate()}
              >
                {logoutMutation.isPending ? 'Logging out...' : 'Log out'}
              </button>
            </div>
          </form>
        </PageCard>
      ) : (
        <PageCard
          eyebrow="Member Management"
          title="Manage members"
          description="Admin can update member display names or set temporary replacement passwords."
        >
          {membersQuery.isLoading ? (
            <p className="empty-panel-note">Loading members...</p>
          ) : null}
          {membersQuery.isError ? (
            <p className="error">Could not load member management.</p>
          ) : null}
          <div className="member-list">
            {(membersQuery.data?.members ?? []).map((member) => (
              <MemberEditor key={member.id} member={member} />
            ))}
          </div>
        </PageCard>
      )}
    </div>
  );
}
