import type { FormEvent } from 'react';

interface JoinMenuModalProps {
  displayName: string;
  error: string | null;
  isJoining: boolean;
  onDisplayNameChange: (value: string) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
}

export default function JoinMenuModal({
  displayName,
  error,
  isJoining,
  onDisplayNameChange,
  onSubmit,
}: JoinMenuModalProps) {
  return (
    <div className="modal-overlay" role="presentation">
      <form
        aria-modal="true"
        aria-labelledby="join-menu-title"
        className="confirm-modal join-menu-modal"
        role="dialog"
        onSubmit={onSubmit}
      >
        <div>
          <p className="card-kicker">Join menu</p>
          <h2 id="join-menu-title">Tell us who is editing</h2>
          <p>
            Enter your name before viewing or changing this menu, so the host can
            see who changed what.
          </p>
        </div>
        <label>
          Your display name
          <input
            required
            autoFocus
            minLength={1}
            pattern=".*\S.*"
            value={displayName}
            placeholder="Grandma Lin"
            onChange={(event) => onDisplayNameChange(event.target.value)}
          />
        </label>
        {error ? <p className="error">{error}</p> : null}
        <button disabled={isJoining} type="submit">
          {isJoining ? 'Joining...' : 'Join menu'}
        </button>
      </form>
    </div>
  );
}
