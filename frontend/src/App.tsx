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
        <span className="brand">LetsOrder</span>
        <nav>
          <Link to="/">Create</Link>
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
    </div>
  );
}
