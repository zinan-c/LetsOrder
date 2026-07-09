import { Link, Route, Routes, useLocation } from 'react-router-dom';
import { useEffect, useMemo, useState, type FormEvent } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import CreateGatheringPage from './pages/CreateGatheringPage';
import HostDashboardPage from './pages/HostDashboardPage';
import GatheringPage from './pages/GatheringPage';
import MenusPage from './pages/MenusPage';
import ReviewPage from './pages/ReviewPage';
import SettingsPage from './pages/SettingsPage';
import JoinGatheringPage from './pages/JoinGatheringPage';
import RequireAuth from './components/RequireAuth';
import { getMe, login, register } from './api/auth';
import type { User } from './types/auth';
import {
  clearAuthSession,
  getAuthToken,
  getCurrentUser,
  setAuthSession,
  USER_CHANGED_EVENT,
} from './utils/user';

function inviteCodeFromPath(pathname: string) {
  const match = pathname.match(/^\/(?:menu|host|review)\/([^/?#]+)/);
  return match?.[1];
}

export default function App() {
  const location = useLocation();
  const queryClient = useQueryClient();
  const [currentUser, setCurrentUser] = useState<User | null>(() => getCurrentUser());
  const [authMode, setAuthMode] = useState<'login' | 'register'>('login');
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [displayName, setDisplayName] = useState('');
  const [generatedPassword, setGeneratedPassword] = useState<string | null>(null);
  const [isCheckingSession, setIsCheckingSession] = useState(() =>
    Boolean(getAuthToken()),
  );
  const inviteCode = useMemo(
    () => inviteCodeFromPath(location.pathname),
    [location.pathname],
  );
  const isAdmin = currentUser?.role === 'admin';
  const isAuthenticated = Boolean(currentUser && getAuthToken());

  const loginMutation = useMutation({
    mutationFn: () => login(username.trim(), password),
    onSuccess: (response) => {
      setAuthSession(response.token, response.user);
      setCurrentUser(response.user);
      setPassword('');
      setGeneratedPassword(null);
    },
  });
  const registerMutation = useMutation({
    mutationFn: () => register(displayName.trim(), undefined, inviteCode),
    onSuccess: (response) => {
      setAuthSession(response.token, response.user);
      setCurrentUser(response.user);
      setUsername(response.user.username);
      setDisplayName(response.user.display_name);
      setGeneratedPassword(response.generated_password ?? null);
    },
  });

  useEffect(() => {
    function handleUserChanged(event: Event) {
      const user = event instanceof CustomEvent ? (event.detail as User | null) : null;
      setCurrentUser(user);
      setIsCheckingSession(false);
    }

    window.addEventListener(USER_CHANGED_EVENT, handleUserChanged);

    return () => {
      window.removeEventListener(USER_CHANGED_EVENT, handleUserChanged);
    };
  }, []);

  useEffect(() => {
    const token = getAuthToken();
    if (!token) {
      setCurrentUser(null);
      setIsCheckingSession(false);
      return;
    }

    let ignore = false;

    async function validateSession() {
      setIsCheckingSession(true);
      try {
        const response = await getMe();
        if (ignore) {
          return;
        }

        setAuthSession(token, response.user);
        setCurrentUser(response.user);
      } catch {
        if (!ignore) {
          clearAuthSession();
          setCurrentUser(null);
        }
      } finally {
        if (!ignore) {
          setIsCheckingSession(false);
        }
      }
    }

    validateSession();

    return () => {
      ignore = true;
    };
  }, []);

  useEffect(() => {
    const token = getAuthToken();
    if (!currentUser || !token) {
      return;
    }

    const apiBaseUrl = import.meta.env.VITE_API_BASE_URL ?? window.location.origin;
    const wsUrl = new URL('/api/ws', apiBaseUrl || window.location.origin);
    wsUrl.protocol = wsUrl.protocol === 'https:' ? 'wss:' : 'ws:';
    wsUrl.searchParams.set('token', token);

    const socket = new WebSocket(wsUrl);
    socket.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data) as {
          event?: string;
          gathering_id?: string | null;
        };

        if (message.event !== 'refresh') {
          return;
        }

        queryClient.invalidateQueries({ queryKey: ['gatherings'] });
        queryClient.invalidateQueries({ queryKey: ['gathering'] });

        if (message.gathering_id) {
          queryClient.invalidateQueries({
            queryKey: ['menu-items', message.gathering_id],
          });
          queryClient.invalidateQueries({
            queryKey: ['participants', message.gathering_id],
          });
          queryClient.invalidateQueries({
            queryKey: ['activity-logs', message.gathering_id],
          });
          queryClient.invalidateQueries({
            queryKey: ['photos', message.gathering_id],
          });
        }
      } catch {
        return;
      }
    };

    return () => {
      socket.close();
    };
  }, [currentUser, queryClient]);

  function handleLogin(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    loginMutation.mutate();
  }

  function handleRegister(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!displayName.trim()) {
      return;
    }

    registerMutation.mutate();
  }

  return (
    <div className="app-shell">
      <header className="site-header">
        <span className="brand">LetsOrder</span>
        <nav>
          {isAdmin ? <Link to="/">Initiate</Link> : null}
          <Link to="/menus">Menus</Link>
          {currentUser ? <Link to="/settings">Setting</Link> : null}
        </nav>
      </header>

      <main>
        <Routes>
          <Route
            path="/"
            element={
              <RequireAuth
                isAuthenticated={isAuthenticated}
                isCheckingSession={isCheckingSession}
              >
                <CreateGatheringPage />
              </RequireAuth>
            }
          />
          <Route
            path="/join"
            element={
              <RequireAuth
                isAuthenticated={isAuthenticated}
                isCheckingSession={isCheckingSession}
              >
                <JoinGatheringPage />
              </RequireAuth>
            }
          />
          <Route
            path="/menus"
            element={
              <RequireAuth
                isAuthenticated={isAuthenticated}
                isCheckingSession={isCheckingSession}
              >
                <MenusPage />
              </RequireAuth>
            }
          />
          <Route
            path="/menu/:inviteCode"
            element={
              <RequireAuth
                isAuthenticated={isAuthenticated}
                isCheckingSession={isCheckingSession}
              >
                <GatheringPage />
              </RequireAuth>
            }
          />
          <Route
            path="/host/:inviteCode"
            element={
              <RequireAuth
                isAuthenticated={isAuthenticated}
                isCheckingSession={isCheckingSession}
              >
                <HostDashboardPage />
              </RequireAuth>
            }
          />
          <Route
            path="/review/:inviteCode"
            element={
              <RequireAuth
                isAuthenticated={isAuthenticated}
                isCheckingSession={isCheckingSession}
              >
                <ReviewPage />
              </RequireAuth>
            }
          />
          <Route
            path="/settings"
            element={
              <RequireAuth
                isAuthenticated={isAuthenticated}
                isCheckingSession={isCheckingSession}
              >
                <SettingsPage />
              </RequireAuth>
            }
          />
        </Routes>
      </main>

      {!currentUser || generatedPassword ? (
        <div className="modal-overlay" role="presentation">
          {generatedPassword && currentUser ? (
            <section
              aria-modal="true"
              aria-labelledby="generated-password-title"
              className="confirm-modal join-menu-modal"
              role="dialog"
            >
              <div>
                <p className="card-kicker">Account ready</p>
                <h2 id="generated-password-title">Save your login</h2>
                <p>Your password is generated from your name plus three random digits.</p>
              </div>
              <div className="result-panel">
                <strong>Username: {username}</strong>
                <strong>Password: {generatedPassword}</strong>
              </div>
              <button type="button" onClick={() => setGeneratedPassword(null)}>
                Continue
              </button>
            </section>
          ) : authMode === 'login' ? (
            <form
              aria-modal="true"
              aria-labelledby="login-title"
              className="confirm-modal join-menu-modal"
              role="dialog"
              onSubmit={handleLogin}
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
                  onChange={(event) => setUsername(event.target.value)}
                />
              </label>
              <label>
                Password
                <input
                  required
                  type="password"
                  value={password}
                  placeholder="Password"
                  onChange={(event) => setPassword(event.target.value)}
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
                onClick={() => setAuthMode('register')}
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
              onSubmit={handleRegister}
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
                  onChange={(event) => setDisplayName(event.target.value)}
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
                onClick={() => setAuthMode('login')}
              >
                Back to login
              </button>
            </form>
          )}
        </div>
      ) : null}
    </div>
  );
}
