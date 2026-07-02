import { Link, useParams } from 'react-router-dom';
import PageCard from '../components/PageCard';

export default function GatheringPage() {
  const { inviteCode } = useParams();

  return (
    <PageCard
      eyebrow={`Invite code: ${inviteCode}`}
      title="Collaborative menu"
      description="This page will become the shared menu editor for participants."
    >
      <div className="placeholder-list">
        <p>Next backend endpoints to wire here:</p>
        <ul>
          <li>Join as participant</li>
          <li>Load gathering by invite code</li>
          <li>Add and edit menu items</li>
          <li>Show locked state after expiration</li>
        </ul>
      </div>
      <Link className="button-link" to={`/g/${inviteCode}/review`}>
        Open review page
      </Link>
    </PageCard>
  );
}
