import { Link, useParams } from 'react-router-dom';
import DishCard from '../components/DishCard';
import GatheringSummary from '../components/GatheringSummary';
import PageCard from '../components/PageCard';
import StatusPill from '../components/StatusPill';
import { mockMenuItems } from '../data/mockGathering';

export default function GatheringPage() {
  const { inviteCode } = useParams();
  const activeItems = mockMenuItems.filter((item) => item.status !== 'cancelled');
  const cancelledItems = mockMenuItems.filter(
    (item) => item.status === 'cancelled',
  );

  return (
    <div className="workspace-grid">
      <section>
        <div className="page-heading-row">
          <div>
            <p className="eyebrow">Invite code: {inviteCode}</p>
            <h1>Menu workspace</h1>
            <p className="lead">
              Add dishes, claim prep work, and keep the family menu tidy before
              it locks.
            </p>
          </div>
          <button type="button">Add dish</button>
        </div>

        <div className="toolbar">
          <StatusPill>All</StatusPill>
          <StatusPill tone="green">Prepared</StatusPill>
          <StatusPill tone="red">Cancelled</StatusPill>
          <span className="toolbar-note">Menu editing locks tomorrow at 6 PM</span>
        </div>

        <div className="dish-grid">
          {activeItems.map((item) => (
            <DishCard item={item} key={item.id} />
          ))}
        </div>

        <PageCard
          eyebrow="No deleting, just history"
          title="Cancelled items stay visible"
          description="For family planning, knowing what changed is useful. Cancelled dishes stay as a soft history trail."
        >
          <div className="dish-grid compact-grid">
            {cancelledItems.map((item) => (
              <DishCard item={item} key={item.id} />
            ))}
          </div>
        </PageCard>
      </section>

      <aside className="sticky-side">
        <GatheringSummary />
        <div className="edit-drawer-preview">
          <p className="card-kicker">Edit dish drawer</p>
          <h2>Add or update a dish</h2>
          <label>
            Dish name
            <input placeholder="Crispy tofu" />
          </label>
          <div className="split-fields">
            <label>
              Qty
              <input placeholder="2" />
            </label>
            <label>
              Unit
              <input placeholder="plates" />
            </label>
          </div>
          <label>
            Status
            <select defaultValue="planned">
              <option value="planned">Planned</option>
              <option value="prepared">Prepared</option>
              <option value="cancelled">Cancelled</option>
            </select>
          </label>
          <label>
            Notes
            <textarea placeholder="Any prep details?" />
          </label>
          <button type="button">Save item</button>
        </div>
        <Link className="button-link secondary full-width" to={`/g/${inviteCode}/review`}>
          Open review
        </Link>
      </aside>
    </div>
  );
}
