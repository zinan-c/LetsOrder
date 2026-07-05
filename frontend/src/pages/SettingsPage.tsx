import { FormEvent, useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { logout, updateAccount } from '../api/auth';
import PageCard from '../components/PageCard';
import { clearAuthSession, getCurrentUser, updateStoredUser } from '../utils/user';

export default function SettingsPage() {
  const currentUser = getCurrentUser();
  const isSystemAdmin = currentUser?.username === 'suite-admin';
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
        <button className="settings-nav-item active" type="button">
          Account Setting
        </button>
      </aside>

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
              System admin uses the fixed password `Admin_1234`.
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
    </div>
  );
}
