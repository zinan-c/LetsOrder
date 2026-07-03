import { useQuery } from '@tanstack/react-query';
import { Link, useParams } from 'react-router-dom';
import { getGatheringByInviteCode } from '../api/gatherings';
import { listMenuItems } from '../api/menuItems';
import DishCard from '../components/DishCard';
import PageCard from '../components/PageCard';
import StatusPill from '../components/StatusPill';
import { mockPhotos } from '../data/mockGathering';

export default function ReviewPage() {
  const { inviteCode } = useParams();
  const gatheringQuery = useQuery({
    queryKey: ['gathering', inviteCode],
    queryFn: () => getGatheringByInviteCode(inviteCode ?? ''),
    enabled: Boolean(inviteCode),
    retry: false,
  });
  const gathering = gatheringQuery.data?.gathering;
  const menuItemsQuery = useQuery({
    queryKey: ['menu-items', gathering?.id],
    queryFn: () => listMenuItems(gathering?.id ?? ''),
    enabled: Boolean(gathering?.id && gathering?.is_locked),
    retry: false,
  });
  const finalMenuItems =
    menuItemsQuery.data?.menu_items.filter((item) => item.status !== 'cancelled') ??
    [];

  if (gathering && !gathering.is_locked) {
    return (
      <PageCard
        eyebrow="Gathering archive"
        title="Review is not ready yet"
        description="The final menu becomes available after this gathering is locked."
      >
        <div className="action-row">
          <Link className="button-link secondary" to={`/menu/${inviteCode}`}>
            Back to menu
          </Link>
          <Link className="button-link secondary" to={`/host/${inviteCode}`}>
            Host controls
          </Link>
        </div>
      </PageCard>
    );
  }

  return (
    <div className="review-layout">
      <PageCard
        eyebrow="Gathering archive"
        title={gathering ? `${gathering.title} review` : 'Review'}
        description="After the menu locks, this page keeps the final menu and photo memories together."
      >
        <div className="action-row">
          <StatusPill tone="neutral">Read-only menu</StatusPill>
          <Link className="button-link secondary" to={`/menu/${inviteCode}`}>
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
        <div className="dish-list final-menu-list">
          {finalMenuItems.map((item) => (
            <DishCard item={item} key={item.id} readOnly />
          ))}
          {finalMenuItems.length === 0 ? (
            <p className="empty-panel-note">No final menu items yet.</p>
          ) : null}
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
