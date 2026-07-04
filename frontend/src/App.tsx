import { Link, Route, Routes, useLocation } from 'react-router-dom';
import { useEffect, useState } from 'react';
import CreateGatheringPage from './pages/CreateGatheringPage';
import HostDashboardPage from './pages/HostDashboardPage';
import GatheringPage from './pages/GatheringPage';
import MenusPage from './pages/MenusPage';
import ReviewPage from './pages/ReviewPage';
import { syncUserFromQuery } from './utils/user';

export default function App() {
  const location = useLocation();
  const [currentUser, setCurrentUser] = useState(() =>
    syncUserFromQuery(window.location.search),
  );
  const isAdmin = currentUser === 'admin';

  useEffect(() => {
    setCurrentUser(syncUserFromQuery(location.search));
  }, [location.search]);

  return (
    <div className="app-shell">
      <header className="site-header">
        <span className="brand">LetsOrder</span>
        {isAdmin ? (
          <nav>
            <Link to="/">Initiate</Link>
            <Link to="/menus">Menus</Link>
          </nav>
        ) : null}
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
    </div>
  );
}
