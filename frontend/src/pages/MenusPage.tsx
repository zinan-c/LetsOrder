import { useQuery } from '@tanstack/react-query';
import { Link } from 'react-router-dom';
import { listGatherings } from '../api/gatherings';
import PageCard from '../components/PageCard';
import StatusPill from '../components/StatusPill';
import { mockGathering, mockMenuItems } from '../data/mockGathering';

const fallbackMenus = [
  {
    id: 'mock-menu',
    title: mockGathering.title,
    description: mockGathering.description,
    invite_code: mockGathering.inviteCode,
    status: 'active',
    item_count: mockMenuItems.length,
    prepared_count: mockMenuItems.filter((item) => item.status === 'prepared').length,
    participant_count: mockGathering.participantCount,
  },
];

export default function MenusPage() {
  const gatheringsQuery = useQuery({
    queryKey: ['gatherings'],
    queryFn: listGatherings,
    retry: false,
  });
  const menus = gatheringsQuery.data?.gatherings.length
    ? gatheringsQuery.data.gatherings
    : fallbackMenus;
  const isUsingFallback = !gatheringsQuery.data?.gatherings.length;

  return (
    <PageCard
      eyebrow="Menus"
      title="Choose a menu to work on"
      description="Browse family gathering menus, then open one to edit dishes, claim prep work, and review what changed."
    >
      <div className="menu-list">
        {menus.map((menu) => (
          <Link
            className="menu-list-row"
            key={menu.id}
            to={`/api/menu/${menu.invite_code}`}
          >
            <div>
              <p className="card-kicker">
                {isUsingFallback ? 'Prototype menu' : 'Menu'}
              </p>
              <h2>{menu.title}</h2>
              <p>{menu.description}</p>
            </div>
            <div className="menu-list-meta">
              <StatusPill>{menu.status}</StatusPill>
              <strong>{menu.item_count} items</strong>
              <span>{menu.prepared_count} prepared</span>
              <span>{menu.participant_count} people</span>
            </div>
          </Link>
        ))}
      </div>
      {gatheringsQuery.isError ? (
        <p className="error">
          Could not load menus from the API. Showing prototype data.
        </p>
      ) : null}
    </PageCard>
  );
}
