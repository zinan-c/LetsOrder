import { Link, useParams } from 'react-router-dom';
import PageCard from '../components/PageCard';

export default function ReviewPage() {
  const { inviteCode } = useParams();

  return (
    <PageCard
      eyebrow="Gathering archive"
      title="Review the final menu and photos"
      description="After the menu locks, this page keeps the final menu and photo memories together."
    >
      <div className="placeholder-list">
        <p>Planned review features:</p>
        <ul>
          <li>Read-only final menu</li>
          <li>Photo uploads</li>
          <li>Captions and upload history</li>
        </ul>
      </div>
      <Link className="button-link" to={`/g/${inviteCode}`}>
        Back to menu
      </Link>
    </PageCard>
  );
}
