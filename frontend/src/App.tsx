import { Link, Route, Routes } from 'react-router-dom';
import CreateGatheringPage from './pages/CreateGatheringPage';
import HostDashboardPage from './pages/HostDashboardPage';
import InviteLandingPage from './pages/InviteLandingPage';
import GatheringPage from './pages/GatheringPage';
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
          <Link to="/g/hotpot-8f3a/menu">Menu</Link>
          <Link to="/g/hotpot-8f3a/host">Host</Link>
          <Link to="/g/hotpot-8f3a/review">Review</Link>
        </nav>
      </header>

      <main>
        <Routes>
          <Route path="/" element={<CreateGatheringPage />} />
          <Route path="/g/:inviteCode" element={<InviteLandingPage />} />
          <Route path="/g/:inviteCode/menu" element={<GatheringPage />} />
          <Route path="/g/:inviteCode/host" element={<HostDashboardPage />} />
          <Route path="/g/:inviteCode/review" element={<ReviewPage />} />
        </Routes>
      </main>
    </div>
  );
}
