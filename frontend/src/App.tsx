import { Link, Route, Routes } from 'react-router-dom';
import CreateGatheringPage from './pages/CreateGatheringPage';
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
        </nav>
      </header>

      <main>
        <Routes>
          <Route path="/" element={<CreateGatheringPage />} />
          <Route path="/g/:inviteCode" element={<GatheringPage />} />
          <Route path="/g/:inviteCode/review" element={<ReviewPage />} />
        </Routes>
      </main>
    </div>
  );
}
