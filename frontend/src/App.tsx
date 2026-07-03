import { Link, Route, Routes } from 'react-router-dom';
import CreateGatheringPage from './pages/CreateGatheringPage';
import HostDashboardPage from './pages/HostDashboardPage';
import GatheringPage from './pages/GatheringPage';
import MenusPage from './pages/MenusPage';
import ReviewPage from './pages/ReviewPage';

export default function App() {
  return (
    <div className="app-shell">
      <header className="site-header">
        <Link className="brand" to="/">
          LetsOrder
        </Link>
        <nav>
          <Link to="/">Create</Link>
          <Link to="/api/menus">Menus</Link>
        </nav>
      </header>

      <main>
        <Routes>
          <Route path="/" element={<CreateGatheringPage />} />
          <Route path="/api/menus" element={<MenusPage />} />
          <Route path="/api/menu/:inviteCode" element={<GatheringPage />} />
          <Route path="/api/host/:inviteCode" element={<HostDashboardPage />} />
          <Route path="/api/review/:inviteCode" element={<ReviewPage />} />
        </Routes>
      </main>
    </div>
  );
}
