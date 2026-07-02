import { FormEvent, useState } from 'react';
import { Link, useParams } from 'react-router-dom';
import GatheringSummary from '../components/GatheringSummary';
import PageCard from '../components/PageCard';
import { mockMenuItems } from '../data/mockGathering';

export default function InviteLandingPage() {
  const { inviteCode } = useParams();
  const [displayName, setDisplayName] = useState('');

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
  }

  return (
    <div className="two-column">
      <PageCard
        eyebrow="You are invited"
        title="Join this family menu board"
        description="Add your name first. Then you can help add dishes, update prep notes, and see what everyone is bringing."
      >
        <form className="form-grid compact" onSubmit={handleSubmit}>
          <label>
            Your display name
            <input
              required
              value={displayName}
              onChange={(event) => setDisplayName(event.target.value)}
              placeholder="Grandma Lin"
            />
          </label>
          <Link className="button-link" to={`/g/${inviteCode}/menu`}>
            Join menu
          </Link>
        </form>

        <div className="mini-menu-preview">
          <p className="card-kicker">Current menu preview</p>
          {mockMenuItems.slice(0, 3).map((item) => (
            <div className="mini-row" key={item.id}>
              <span>{item.name}</span>
              <strong>
                {item.quantity} {item.unit}
              </strong>
            </div>
          ))}
        </div>
      </PageCard>
      <GatheringSummary />
    </div>
  );
}
