import type { FormEvent } from 'react';
import type { UseMutationResult } from '@tanstack/react-query';
import type { AuthResponse } from '../types/auth';

interface AuthModalProps {
  authMode: 'login' | 'register';
  displayName: string;
  generatedPassword: string | null;
  loginMutation: UseMutationResult<AuthResponse, Error, void, unknown>;
  password: string;
  registerMutation: UseMutationResult<AuthResponse, Error, void, unknown>;
  username: string;
  onAuthModeChange: (mode: 'login' | 'register') => void;
  onContinueGeneratedPassword: () => void;
  onDisplayNameChange: (value: string) => void;
  onLogin: (event: FormEvent<HTMLFormElement>) => void;
  onPasswordChange: (value: string) => void;
  onRegister: (event: FormEvent<HTMLFormElement>) => void;
  onUsernameChange: (value: string) => void;
}

export default function AuthModal({
  authMode,
  displayName,
  generatedPassword,
  loginMutation,
  password,
  registerMutation,
  username,
  onAuthModeChange,
  onContinueGeneratedPassword,
  onDisplayNameChange,
  onLogin,
  onPasswordChange,
  onRegister,
  onUsernameChange,
}: AuthModalProps) {
  return (
    <div className="modal-overlay" role="presentation">
      {generatedPassword ? (
        <section
          aria-modal="true"
          aria-labelledby="generated-password-title"
          className="confirm-modal join-menu-modal"
          role="dialog"
        >
          <div>
            <p className="card-kicker">Account ready</p>
            <h2 id="generated-password-title">Save your login</h2>
            <p>Your temporary password was generated for this account.</p>
          </div>
          <div className="result-panel">
            <strong>Username: {username}</strong>
            <strong>Password: {generatedPassword}</strong>
          </div>
          <button type="button" onClick={onContinueGeneratedPassword}>
            Continue
          </button>
        </section>
      ) : authMode === 'login' ? (
        <form
          aria-modal="true"
          aria-labelledby="login-title"
          className="confirm-modal join-menu-modal"
          role="dialog"
          onSubmit={onLogin}
        >
          <div>
            <p className="card-kicker">Welcome back</p>
            <h2 id="login-title">Log in to LetsOrder</h2>
            <p>Use your account before viewing or changing gathering menus.</p>
          </div>
          <label>
            Username
            <input
              required
              autoFocus
              value={username}
              placeholder="suite-admin"
              onChange={(event) => onUsernameChange(event.target.value)}
            />
          </label>
          <label>
            Password
            <input
              required
              type="password"
              value={password}
              placeholder="Password"
              onChange={(event) => onPasswordChange(event.target.value)}
            />
          </label>
          {loginMutation.isError ? (
            <p className="error">Login failed. Please check your username and password.</p>
          ) : null}
          <button disabled={loginMutation.isPending} type="submit">
            {loginMutation.isPending ? 'Logging in...' : 'Log in'}
          </button>
          <button
            className="ghost-button"
            type="button"
            onClick={() => onAuthModeChange('register')}
          >
            First time? Click here
          </button>
        </form>
      ) : (
        <form
          aria-modal="true"
          aria-labelledby="register-title"
          className="confirm-modal join-menu-modal"
          role="dialog"
          onSubmit={onRegister}
        >
          <div>
            <p className="card-kicker">First time</p>
            <h2 id="register-title">Tell us who you are</h2>
            <p>Enter your name and we will create your login for this gathering.</p>
          </div>
          <label>
            Your name
            <input
              required
              autoFocus
              value={displayName}
              placeholder="Grandma Lin"
              onChange={(event) => onDisplayNameChange(event.target.value)}
            />
          </label>
          {registerMutation.isError ? (
            <p className="error">Could not create your account.</p>
          ) : null}
          <button disabled={registerMutation.isPending} type="submit">
            {registerMutation.isPending ? 'Creating...' : 'Create account'}
          </button>
          <button
            className="ghost-button"
            type="button"
            onClick={() => onAuthModeChange('login')}
          >
            Back to login
          </button>
        </form>
      )}
    </div>
  );
}
