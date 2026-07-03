import { Link, useParams } from 'react-router-dom';
import DishCard from '../components/DishCard';
import PageCard from '../components/PageCard';
import StatusPill from '../components/StatusPill';
import { mockMenuItems, mockPhotos } from '../data/mockGathering';

export default function ReviewPage() {
  const { inviteCode } = useParams();

  return (
    <div className="review-layout">
      <PageCard
        eyebrow="Gathering archive"
        title="Review the final menu and photos"
        description="After the menu locks, this page keeps the final menu and photo memories together."
      >
        <div className="action-row">
          <StatusPill tone="neutral">Read-only menu</StatusPill>
          <Link className="button-link secondary" to={`/api/menu/${inviteCode}`}>
            Back to menu
          </Link>
        </div>
      </PageCard>

      <section className="section-block">
        <div className="panel-header">
          <div>
            <p className="card-kicker">Final menu</p>
            <h2>What made it to the table</h2>
          </div>
        </div>
        <div className="dish-grid">
          {mockMenuItems
            .filter((item) => item.status !== 'cancelled')
            .map((item) => (
              <DishCard item={item} key={item.id} readOnly />
            ))}
        </div>
      </section>

      <section className="section-block">
        <div className="panel-header">
          <div>
            <p className="card-kicker">Photo wall</p>
            <h2>Little memories, neatly kept</h2>
          </div>
          <button type="button">Upload photos</button>
        </div>
        <div className="photo-grid">
          {mockPhotos.map((photo) => (
            <article className={`photo-card photo-${photo.color}`} key={photo.id}>
              <div className="photo-blob" />
              <p>{photo.title}</p>
            </article>
          ))}
          <article className="upload-card">
            <strong>Drop photos here</strong>
            <span>JPG, PNG, or HEIC later</span>
          </article>
        </div>
      </section>
    </div>
  );
}
