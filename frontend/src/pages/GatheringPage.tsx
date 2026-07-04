import { FormEvent, useEffect, useMemo, useState } from 'react';
import { useParams } from 'react-router-dom';
import {
  getGatheringByInviteCode,
  joinGathering,
  listParticipants,
} from '../api/gatherings';
import {
  createMenuItem,
  listMenuItems,
  updateMenuItem,
} from '../api/menuItems';
import DishCard from '../components/DishCard';
import GatheringSummary from '../components/GatheringSummary';
import StatusPill from '../components/StatusPill';
import { mockGathering, mockMenuItems } from '../data/mockGathering';
import type { Gathering, Participant } from '../types/gathering';
import type { MenuItem, MenuItemStatus } from '../types/menu';
import { formatDateTime } from '../utils/dateTime';
import { getCookieUser, setCookieUser } from '../utils/user';

function participantStorageKey(inviteCode?: string) {
  return `letsorder:${inviteCode ?? 'unknown'}:participant_id`;
}

const categoryOptions = [
  'Main',
  'Protein',
  'Vegetables',
  'Snack',
  'Dessert',
  'Drink',
  'Other',
];

const unitOptions = [
  'plates',
  'boxes',
  'cups',
  'pot',
  'servings',
  'pieces',
  'bags',
];

function createLocalMenuItem(
  formData: FormData,
  inviteCode?: string,
  editingItem?: MenuItem | null,
): MenuItem {
  const now = new Date().toISOString();
  const quantity = Number(formData.get('quantity') || 1);

  return {
    id: editingItem?.id ?? `local-${crypto.randomUUID()}`,
    gathering_id: editingItem?.gathering_id ?? `local-${inviteCode ?? 'gathering'}`,
    created_by: editingItem?.created_by ?? 'local-participant',
    updated_by: editingItem ? 'local-participant' : null,
    name: String(formData.get('name') ?? '').trim(),
    category: String(formData.get('category') ?? '').trim() || null,
    quantity: Number.isFinite(quantity) && quantity > 0 ? quantity : 1,
    unit: String(formData.get('unit') ?? '').trim() || null,
    owner_name: String(formData.get('owner_name') ?? '').trim() || null,
    note: String(formData.get('note') ?? '').trim() || null,
    status: String(formData.get('status') ?? 'planned') as MenuItemStatus,
    created_at: editingItem?.created_at ?? now,
    updated_at: now,
  };
}

export default function GatheringPage() {
  const { inviteCode } = useParams();
  const [editingItem, setEditingItem] = useState<MenuItem | null>(null);
  const [isEditorOpen, setIsEditorOpen] = useState(false);
  const [menuItems, setMenuItems] = useState<MenuItem[]>(mockMenuItems);
  const [currentGathering, setCurrentGathering] = useState<Gathering | null>(null);
  const [participants, setParticipants] = useState<Participant[]>([]);
  const [gatheringId, setGatheringId] = useState<string | null>(null);
  const [currentUser, setCurrentUser] = useState(() => getCookieUser());
  const [displayName, setDisplayName] = useState(() => getCookieUser());
  const [participantId, setParticipantId] = useState<string | null>(() =>
    localStorage.getItem(participantStorageKey(inviteCode)),
  );
  const [joinError, setJoinError] = useState<string | null>(null);
  const [isJoining, setIsJoining] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const [statusFilter, setStatusFilter] = useState<'all' | MenuItemStatus>('all');
  const filteredMenuItems = menuItems.filter((item) =>
    statusFilter === 'all' ? true : item.status === statusFilter,
  );
  const sortedMenuItems = [...filteredMenuItems].sort((left, right) => {
    if (left.status === right.status) {
      return 0;
    }

    if (left.status === 'cancelled') {
      return 1;
    }

    if (right.status === 'cancelled') {
      return -1;
    }

    return 0;
  });
  const isEditing = Boolean(editingItem);
  const canUseApi = Boolean(gatheringId && participantId);

  useEffect(() => {
    if (!inviteCode) {
      return;
    }

    const currentInviteCode = inviteCode;
    let ignore = false;

    async function loadMenuItems() {
      try {
        const gatheringResponse = await getGatheringByInviteCode(currentInviteCode);
        if (ignore) {
          return;
        }

        setCurrentGathering(gatheringResponse.gathering);
        setGatheringId(gatheringResponse.gathering.id);

        let storedParticipantId = localStorage.getItem(
          participantStorageKey(currentInviteCode),
        );
        const participantsResponse = await listParticipants(
          gatheringResponse.gathering.id,
        );
        if (!ignore) {
          setParticipants(participantsResponse.participants);
        }

        if (storedParticipantId && !currentUser) {
          const storedParticipant = participantsResponse.participants.find(
            (participant) => participant.id === storedParticipantId,
          );

          if (storedParticipant) {
            setCurrentUser(storedParticipant.display_name);
            setDisplayName(storedParticipant.display_name);
            setCookieUser(storedParticipant.display_name);
          }
        }

        if (!storedParticipantId && currentUser) {
          const joinResponse = await joinGathering(
            gatheringResponse.gathering.id,
            currentUser,
          );
          storedParticipantId = joinResponse.participant.id;
          localStorage.setItem(
            participantStorageKey(currentInviteCode),
            joinResponse.participant.id,
          );
          localStorage.setItem(
            `letsorder:${currentInviteCode}:access_token`,
            joinResponse.access_token,
          );
          if (!ignore) {
            setParticipants((items) => [joinResponse.participant, ...items]);
          }
        }

        setParticipantId(storedParticipantId);

        const menuResponse = await listMenuItems(gatheringResponse.gathering.id);
        if (!ignore) {
          setMenuItems(menuResponse.menu_items);
        }
      } catch {
        if (!ignore) {
          setGatheringId(null);
          setCurrentGathering(null);
          setParticipants([]);
          setMenuItems(mockMenuItems);
        }
      }
    }

    loadMenuItems();

    return () => {
      ignore = true;
    };
  }, [currentUser, inviteCode]);

  const editorModeLabel = useMemo(() => {
    if (canUseApi) {
      return 'Connected to API';
    }

    return 'Local prototype mode';
  }, [canUseApi]);

  const ownerOptions = useMemo(() => {
    const names = new Set(
      participants
        .map((participant) => participant.display_name)
        .filter(Boolean),
    );

    if (currentUser) {
      names.add(currentUser);
    }

    if (editingItem?.owner_name) {
      names.add(editingItem.owner_name);
    }

    return [...names];
  }, [currentUser, editingItem?.owner_name, participants]);

  async function handleJoinMenu(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (!gatheringId || !inviteCode) {
      return;
    }

    const name = displayName.trim();
    if (!name) {
      setJoinError('Your display name is required.');
      return;
    }

    setIsJoining(true);
    setJoinError(null);

    try {
      const response = await joinGathering(gatheringId, name);
      localStorage.setItem(
        participantStorageKey(inviteCode),
        response.participant.id,
      );
      localStorage.setItem(
        `letsorder:${inviteCode}:access_token`,
        response.access_token,
      );
      setCookieUser(name);
      setCurrentUser(name);
      setParticipantId(response.participant.id);
      setParticipants((items) => [response.participant, ...items]);
    } catch (error) {
      setJoinError(error instanceof Error ? error.message : 'Failed to join menu.');
    } finally {
      setIsJoining(false);
    }
  }

  function openAddDish() {
    if (isCurrentMenuLocked || (gatheringId && !participantId)) {
      return;
    }

    setEditingItem(null);
    setSaveError(null);
    setIsEditorOpen(true);
  }

  function openEditDish(item: MenuItem) {
    if (isCurrentMenuLocked || (gatheringId && !participantId)) {
      return;
    }

    setEditingItem(item);
    setSaveError(null);
    setIsEditorOpen(true);
  }

  function closeEditor() {
    setIsEditorOpen(false);
    setEditingItem(null);
  }

  async function handleSaveMenuItem(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    const formData = new FormData(event.currentTarget);
    const name = String(formData.get('name') ?? '').trim();

    if (!name) {
      setSaveError('Dish name is required.');
      return;
    }

    setIsSaving(true);
    setSaveError(null);

    try {
      if (canUseApi && gatheringId && participantId) {
        if (editingItem) {
          const response = await updateMenuItem(editingItem.id, {
            updated_by: participantId,
            name,
            category: String(formData.get('category') ?? '').trim(),
            quantity: Number(formData.get('quantity') || 1),
            unit: String(formData.get('unit') ?? '').trim(),
            owner_name:
              String(formData.get('owner_name') ?? '').trim() || currentUser,
            note: String(formData.get('note') ?? '').trim(),
            status: String(formData.get('status') ?? 'planned') as MenuItemStatus,
          });

          setMenuItems((items) =>
            items.map((item) =>
              item.id === response.menu_item.id ? response.menu_item : item,
            ),
          );
        } else {
          const response = await createMenuItem(gatheringId, {
            created_by: participantId,
            name,
            category: String(formData.get('category') ?? '').trim(),
            quantity: Number(formData.get('quantity') || 1),
            unit: String(formData.get('unit') ?? '').trim(),
            owner_name:
              String(formData.get('owner_name') ?? '').trim() || currentUser,
            note: String(formData.get('note') ?? '').trim(),
            status: String(formData.get('status') ?? 'planned') as MenuItemStatus,
          });

          setMenuItems((items) => [...items, response.menu_item]);
        }
      } else {
        const localItem = createLocalMenuItem(formData, inviteCode, editingItem);

        if (editingItem) {
          setMenuItems((items) =>
            items.map((item) => (item.id === editingItem.id ? localItem : item)),
          );
        } else {
          setMenuItems((items) => [...items, localItem]);
        }
      }

      closeEditor();
    } catch (error) {
      setSaveError(
        error instanceof Error ? error.message : 'Failed to save menu item.',
      );
    } finally {
      setIsSaving(false);
    }
  }

  const editorTitle = isEditing ? 'Update this dish' : 'Add a new dish';
  const currentTitle = currentGathering?.title ?? mockGathering.title;
  const currentDescription =
    currentGathering?.description ?? mockGathering.description;
  const currentExpiresAt = currentGathering?.expires_at ?? mockGathering.expiresAt;
  const currentInviteCode =
    currentGathering?.invite_code ?? inviteCode ?? mockGathering.inviteCode;
  const isCurrentMenuLocked = currentGathering?.is_locked ?? false;
  const needsDisplayName = Boolean(currentGathering && !participantId);

  return (
    <div className="menu-workspace">
      <section>
        <div className="page-heading-row">
          <div>
            <p className="eyebrow">Menu workspace · {inviteCode}</p>
            <h1 className="menu-title">{currentTitle}</h1>
            <p className="lead">
              Add dishes, claim prep work, and keep the family menu tidy before
              it locks.
            </p>
          </div>
          <button
            disabled={isCurrentMenuLocked || needsDisplayName}
            type="button"
            onClick={openAddDish}
          >
            {isCurrentMenuLocked
              ? 'Menu locked'
              : needsDisplayName
                ? 'Join first'
                : 'Add dish'}
          </button>
        </div>

        <GatheringSummary
          title={currentTitle}
          description={currentDescription}
          inviteCode={currentInviteCode}
          expiresAt={currentExpiresAt}
          isLocked={isCurrentMenuLocked}
          participantCount={
            currentGathering ? undefined : mockGathering.participantCount
          }
        />

        {needsDisplayName ? (
          <form className="join-panel" onSubmit={handleJoinMenu}>
            <div>
              <p className="card-kicker">Join menu</p>
              <h2>Tell us who is editing</h2>
              <p>
                Enter your name before adding or updating dishes, so the host can
                see who changed what.
              </p>
            </div>
            <label>
              Your display name
              <input
                required
                value={displayName}
                placeholder="Grandma Lin"
                onChange={(event) => setDisplayName(event.target.value)}
              />
            </label>
            {joinError ? <p className="error">{joinError}</p> : null}
            <button disabled={isJoining} type="submit">
              {isJoining ? 'Joining...' : 'Join menu'}
            </button>
          </form>
        ) : null}

        <div className="toolbar">
          <StatusPill tone={canUseApi ? 'green' : 'neutral'}>
            {editorModeLabel}
          </StatusPill>
          <label className="status-filter">
            Status
            <select
              value={statusFilter}
              onChange={(event) =>
                setStatusFilter(event.target.value as 'all' | MenuItemStatus)
              }
            >
              <option value="all">All</option>
              <option value="planned">Planned</option>
              <option value="prepared">Prepared</option>
              <option value="cancelled">Cancelled</option>
            </select>
          </label>
          <span className="toolbar-note">
            {isCurrentMenuLocked
              ? 'Menu editing is locked'
              : `Menu locks ${formatDateTime(currentExpiresAt)}`}
          </span>
        </div>

        <div className="dish-list">
          {sortedMenuItems.map((item) => (
            <DishCard
              item={item}
              key={item.id}
              readOnly={isCurrentMenuLocked}
              onEdit={openEditDish}
            />
          ))}
        </div>
      </section>

      {isEditorOpen ? (
        <div className="modal-overlay" role="presentation" onClick={closeEditor}>
          <form
            aria-modal="true"
            className="dish-editor-modal"
            role="dialog"
            aria-labelledby="dish-editor-title"
            onClick={(event) => event.stopPropagation()}
            onSubmit={handleSaveMenuItem}
          >
            <div className="panel-header">
              <div>
                <p className="card-kicker">Dish editor</p>
                <h2 id="dish-editor-title">{editorTitle}</h2>
              </div>
              <button className="icon-button" type="button" onClick={closeEditor}>
                ×
              </button>
            </div>

            <label>
              Dish name
              <input
                key={`name-${editingItem?.id ?? 'new'}`}
                autoFocus
                name="name"
                placeholder="Crispy tofu"
                defaultValue={editingItem?.name ?? ''}
              />
            </label>

            <label>
              Category
              <select
                key={`category-${editingItem?.id ?? 'new'}`}
                name="category"
                defaultValue={editingItem?.category ?? 'Main'}
              >
                {categoryOptions.map((category) => (
                  <option key={category} value={category}>
                    {category}
                  </option>
                ))}
              </select>
            </label>

            <div className="split-fields">
              <label>
                Qty
                <input
                  key={`quantity-${editingItem?.id ?? 'new'}`}
                  name="quantity"
                  placeholder="2"
                  defaultValue={editingItem?.quantity ?? ''}
                />
              </label>
              <label>
                Unit
                <select
                  key={`unit-${editingItem?.id ?? 'new'}`}
                  name="unit"
                  defaultValue={editingItem?.unit ?? 'plates'}
                >
                  {unitOptions.map((unit) => (
                    <option key={unit} value={unit}>
                      {unit}
                    </option>
                  ))}
                </select>
              </label>
            </div>

            <label>
              Status
              <select
                key={`status-${editingItem?.id ?? 'new'}`}
                name="status"
                defaultValue={editingItem?.status ?? 'planned'}
              >
                <option value="planned">Planned</option>
                <option value="prepared">Prepared</option>
                <option value="cancelled">Cancelled</option>
              </select>
            </label>

            <label>
              Owner
              <select
                key={`owner-${editingItem?.id ?? 'new'}`}
                name="owner_name"
                defaultValue={editingItem?.owner_name ?? currentUser}
              >
                {ownerOptions.map((owner) => (
                  <option key={owner} value={owner}>
                    {owner}
                  </option>
                ))}
              </select>
            </label>

            <label>
              Notes
              <textarea
                key={`note-${editingItem?.id ?? 'new'}`}
                name="note"
                placeholder="Any prep details?"
                defaultValue={editingItem?.note ?? ''}
              />
            </label>

            {saveError ? <p className="error">{saveError}</p> : null}

            <div className="action-row modal-actions">
              <button disabled={isSaving} type="submit">
                {isSaving
                  ? 'Saving...'
                  : isEditing
                    ? 'Save changes'
                    : 'Add to menu'}
              </button>
              <button className="ghost-button" type="button" onClick={closeEditor}>
                Cancel
              </button>
            </div>
          </form>
        </div>
      ) : null}
    </div>
  );
}
