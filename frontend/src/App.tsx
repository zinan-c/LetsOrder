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
import AuthModal from './components/AuthModal';
import RequireAuth from './components/RequireAuth';
import { getMe, login, register } from './api/auth';
import type { User } from './types/auth';
import useRealtimeRefresh from './hooks/useRealtimeRefresh';
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
  useRealtimeRefresh(currentUser, queryClient);

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
        <AuthModal
          authMode={authMode}
          displayName={displayName}
          generatedPassword={generatedPassword}
          loginMutation={loginMutation}
          password={password}
          registerMutation={registerMutation}
          username={username}
          onAuthModeChange={setAuthMode}
          onContinueGeneratedPassword={() => setGeneratedPassword(null)}
          onDisplayNameChange={setDisplayName}
          onLogin={handleLogin}
          onPasswordChange={setPassword}
          onRegister={handleRegister}
          onUsernameChange={setUsername}
        />
      ) : null}
    </div>
  );
}
