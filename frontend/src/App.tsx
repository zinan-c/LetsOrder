import { Link, Route, Routes, useLocation } from 'react-router-dom';
import { useEffect, useState, type FormEvent } from 'react';
import CreateGatheringPage from './pages/CreateGatheringPage';
import HostDashboardPage from './pages/HostDashboardPage';
import GatheringPage from './pages/GatheringPage';
import MenusPage from './pages/MenusPage';
import ReviewPage from './pages/ReviewPage';
import { notifyUserChanged, setCookieUser, syncUserFromQuery } from './utils/user';

export default function App() {
  const location = useLocation();
  const [currentUser, setCurrentUser] = useState(() =>
    syncUserFromQuery(window.location.search),
  );
  const [displayName, setDisplayName] = useState(currentUser);
  const isAdmin = currentUser === 'admin';

  useEffect(() => {
    setCurrentUser(syncUserFromQuery(location.search));
  }, [location.search]);

  function handleIdentifyUser(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    const name = displayName.trim();
    if (!name) {
      return;
    }

    setCookieUser(name);
    setCurrentUser(name);
    setDisplayName(name);
    notifyUserChanged(name);
  }

  return (
    <div className="app-shell">
      <header className="site-header">
        <span className="brand">LetsOrder</span>
        <nav>
          {isAdmin ? (
            <Link to="/">Initiate</Link>
          ) : null}
          <Link to="/menus">Menus</Link>
        </nav>
      </header>

      <main>
        <Routes>
          <Route path="/" element={<CreateGatheringPage />} />
          <Route path="/menus" element={<MenusPage />} />
          <Route path="/menu/:inviteCode" element={<GatheringPage />} />
          <Route path="/host/:inviteCode" element={<HostDashboardPage />} />
          <Route path="/review/:inviteCode" element={<ReviewPage />} />
        </Routes>
      </main>

      {!currentUser ? (
        <div className="modal-overlay" role="presentation">
          <form
            aria-modal="true"
            aria-labelledby="identify-user-title"
            className="confirm-modal join-menu-modal"
            role="dialog"
            onSubmit={handleIdentifyUser}
          >
            <div>
              <p className="card-kicker">Welcome</p>
              <h2 id="identify-user-title">Tell us who you are</h2>
              <p>Enter your name before viewing or changing LetsOrder menus.</p>
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
                onChange={(event) => setDisplayName(event.target.value)}
              />
            </label>
            <button type="submit">Continue</button>
          </form>
        </div>
      ) : null}
    </div>
  );
}
