import { useState } from 'react';
import { Link, useParams } from 'react-router-dom';
import DishCard from '../components/DishCard';
import GatheringSummary from '../components/GatheringSummary';
import PageCard from '../components/PageCard';
import StatusPill from '../components/StatusPill';
import { mockMenuItems } from '../data/mockGathering';
import type { MenuItem } from '../types/menu';

export default function GatheringPage() {
  const { inviteCode } = useParams();
  const [editingItem, setEditingItem] = useState<MenuItem | null>(null);
  const [isEditorOpen, setIsEditorOpen] = useState(false);
  const activeItems = mockMenuItems.filter((item) => item.status !== 'cancelled');
  const cancelledItems = mockMenuItems.filter(
    (item) => item.status === 'cancelled',
  );
  const isEditing = Boolean(editingItem);

  function openAddDish() {
    setEditingItem(null);
    setIsEditorOpen(true);
  }

  function openEditDish(item: MenuItem) {
    setEditingItem(item);
    setIsEditorOpen(true);
  }

  function closeEditor() {
    setIsEditorOpen(false);
    setEditingItem(null);
  }

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
          <button type="button" onClick={openAddDish}>
            Add dish
          </button>
        </div>

        <div className="toolbar">
          <StatusPill>All</StatusPill>
          <StatusPill tone="green">Prepared</StatusPill>
          <StatusPill tone="red">Cancelled</StatusPill>
          <span className="toolbar-note">Menu editing locks tomorrow at 6 PM</span>
        </div>

        <div className="dish-grid">
          {activeItems.map((item) => (
            <DishCard item={item} key={item.id} onEdit={openEditDish} />
          ))}
        </div>

        <PageCard
          eyebrow="No deleting, just history"
          title="Cancelled items stay visible"
          description="For family planning, knowing what changed is useful. Cancelled dishes stay as a soft history trail."
        >
          <div className="dish-grid compact-grid">
            {cancelledItems.map((item) => (
              <DishCard item={item} key={item.id} onEdit={openEditDish} />
            ))}
          </div>
        </PageCard>
      </section>

      <aside className="sticky-side">
        <GatheringSummary />
        <div className={`edit-drawer-preview ${isEditorOpen ? 'is-open' : ''}`}>
          <div className="panel-header">
            <div>
              <p className="card-kicker">
                {isEditorOpen ? 'Dish editor' : 'Editor preview'}
              </p>
              <h2>
                {isEditorOpen
                  ? isEditing
                    ? 'Update this dish'
                    : 'Add a new dish'
                  : 'Click Add dish to start'}
              </h2>
            </div>
            {isEditorOpen ? (
              <button className="icon-button" type="button" onClick={closeEditor}>
                ×
              </button>
            ) : null}
          </div>
          {!isEditorOpen ? (
            <p className="dish-note">
              The form opens here in the prototype. Later this becomes a real
              drawer backed by the API.
            </p>
          ) : null}
          <label>
            Dish name
            <input
              key={`name-${editingItem?.id ?? 'new'}`}
              placeholder="Crispy tofu"
              defaultValue={editingItem?.name ?? ''}
              disabled={!isEditorOpen}
            />
          </label>
          <div className="split-fields">
            <label>
              Qty
              <input
                key={`quantity-${editingItem?.id ?? 'new'}`}
                placeholder="2"
                defaultValue={editingItem?.quantity ?? ''}
                disabled={!isEditorOpen}
              />
            </label>
            <label>
              Unit
              <input
                key={`unit-${editingItem?.id ?? 'new'}`}
                placeholder="plates"
                defaultValue={editingItem?.unit ?? ''}
                disabled={!isEditorOpen}
              />
            </label>
          </div>
          <label>
            Status
            <select
              key={`status-${editingItem?.id ?? 'new'}`}
              defaultValue={editingItem?.status ?? 'planned'}
              disabled={!isEditorOpen}
            >
              <option value="planned">Planned</option>
              <option value="prepared">Prepared</option>
              <option value="cancelled">Cancelled</option>
            </select>
          </label>
          <label>
            Notes
            <textarea
              key={`note-${editingItem?.id ?? 'new'}`}
              placeholder="Any prep details?"
              defaultValue={editingItem?.note ?? ''}
              disabled={!isEditorOpen}
            />
          </label>
          <div className="action-row">
            <button disabled={!isEditorOpen} type="button">
              {isEditing ? 'Save changes' : 'Add to menu'}
            </button>
            {isEditorOpen ? (
              <button className="ghost-button" type="button" onClick={closeEditor}>
                Cancel
              </button>
            ) : null}
          </div>
        </div>
        <Link className="button-link secondary full-width" to={`/g/${inviteCode}/review`}>
          Open review
        </Link>
      </aside>
    </div>
  );
}
