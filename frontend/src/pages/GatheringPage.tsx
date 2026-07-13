import { FormEvent, useEffect, useMemo, useRef, useState } from 'react';
import { useLocation, useNavigate, useParams } from 'react-router-dom';
import {
  joinGathering,
  joinGatheringByInviteCode,
  listParticipants,
} from '../api/gatherings';
import {
  createMenuItem,
  listMenuItems,
  updateMenuItem,
  type UpdateMenuItemPayload,
} from '../api/menuItems';
import { ApiError } from '../api/client';
import DishCard from '../components/DishCard';
import GatheringSummary from '../components/GatheringSummary';
import { mockGathering, mockMenuItems } from '../data/mockGathering';
import type { Gathering, Participant } from '../types/gathering';
import type { MenuItem, MenuItemStatus } from '../types/menu';
import { formatDateTime } from '../utils/dateTime';
import {
  getCookieUser,
  setCookieUser,
  USER_CHANGED_EVENT,
} from '../utils/user';

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

interface MenuItemConflict {
  latestItem: MenuItem;
  pendingPayload: UpdateMenuItemPayload;
}

function normalizeReferenceUrl(value: FormDataEntryValue | null) {
  const text = String(value ?? '').trim();
  const url = text
    .split(/\s+/)
    .map((part) => part.replace(/[，,。.!！?？）)】\]]+$/u, ''))
    .find((part) => part.startsWith('http://') || part.startsWith('https://'));

  return url ?? text;
}

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
    reference_url: normalizeReferenceUrl(formData.get('reference_url')) || null,
    note: String(formData.get('note') ?? '').trim() || null,
    status: String(formData.get('status') ?? 'planned') as MenuItemStatus,
    revision: (editingItem?.revision ?? 0) + 1,
    created_at: editingItem?.created_at ?? now,
    updated_at: now,
  };
}

function buildUpdatePayload(
  formData: FormData,
  participantId: string,
  currentUser: string,
  expectedRevision: number,
): UpdateMenuItemPayload {
  return {
    updated_by: participantId,
    name: String(formData.get('name') ?? '').trim(),
    category: String(formData.get('category') ?? '').trim(),
    quantity: Number(formData.get('quantity') || 1),
    unit: String(formData.get('unit') ?? '').trim(),
    owner_name: String(formData.get('owner_name') ?? '').trim() || currentUser,
    reference_url: normalizeReferenceUrl(formData.get('reference_url')),
    note: String(formData.get('note') ?? '').trim(),
    status: String(formData.get('status') ?? 'planned') as MenuItemStatus,
    expected_revision: expectedRevision,
  };
}

export default function GatheringPage() {
  const { inviteCode } = useParams();
  const location = useLocation();
  const navigate = useNavigate();
  const dishEditorFormRef = useRef<HTMLFormElement | null>(null);
  const [editingItem, setEditingItem] = useState<MenuItem | null>(null);
  const [isEditorOpen, setIsEditorOpen] = useState(false);
  const [selectedChef, setSelectedChef] = useState(() => getCookieUser());
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
  const [conflict, setConflict] = useState<MenuItemConflict | null>(null);
  const [statusFilter, setStatusFilter] = useState<'all' | MenuItemStatus>('all');
  const [categoryFilter, setCategoryFilter] = useState('all');
  const categoryFilterOptions = useMemo(
    () =>
      [
        ...new Set(
          menuItems.map((item) => item.category ?? 'Other').filter(Boolean),
        ),
      ].sort(),
    [menuItems],
  );
  const filteredMenuItems = menuItems.filter((item) => {
    const matchesStatus =
      statusFilter === 'all' ? true : item.status === statusFilter;
    const matchesCategory =
      categoryFilter === 'all'
        ? true
        : (item.category ?? 'Other') === categoryFilter;

    return matchesStatus && matchesCategory;
  });
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
  const chefRecommendations = useMemo(() => {
    if (!selectedChef) {
      return [];
    }

    const seenNames = new Set<string>();
    return [...menuItems]
      .filter((item) => item.owner_name === selectedChef)
      .filter((item) => item.status === 'done' || item.status === 'prepared')
      .filter((item) => item.id !== editingItem?.id)
      .sort(
        (left, right) =>
          new Date(right.updated_at).getTime() - new Date(left.updated_at).getTime(),
      )
      .filter((item) => {
        const key = item.name.trim().toLowerCase();
        if (!key || seenNames.has(key)) {
          return false;
        }

        seenNames.add(key);
        return true;
      })
      .slice(0, 8);
  }, [editingItem?.id, menuItems, selectedChef]);
  const isEditing = Boolean(editingItem);
  const canUseApi = Boolean(gatheringId && participantId);
  const isReviewReturn =
    new URLSearchParams(location.search).get('from') === 'review';

  useEffect(() => {
    function handleUserChanged(event: Event) {
      const user = event instanceof CustomEvent ? event.detail : null;
      const name = user?.display_name ?? '';
      setCurrentUser(name);
      setDisplayName(name);
      setSelectedChef(name);
      setParticipantId(null);
    }

    window.addEventListener(USER_CHANGED_EVENT, handleUserChanged);

    return () => {
      window.removeEventListener(USER_CHANGED_EVENT, handleUserChanged);
    };
  }, []);

  useEffect(() => {
    if (!inviteCode) {
      return;
    }

    const currentInviteCode = inviteCode;
    let ignore = false;

    async function loadMenuItems() {
      try {
        const joinResponse = await joinGatheringByInviteCode(currentInviteCode);
        if (ignore) {
          return;
        }

        const gathering = joinResponse.gathering;
        if (!gathering) {
          throw new Error('Missing gathering response.');
        }

        setCurrentGathering(gathering);
        setGatheringId(gathering.id);
        if (joinResponse.participant) {
          setParticipantId(joinResponse.participant.id);
          localStorage.setItem(
            participantStorageKey(currentInviteCode),
            joinResponse.participant.id,
          );
        } else {
          setParticipantId(null);
          localStorage.removeItem(participantStorageKey(currentInviteCode));
        }

        if (gathering.is_locked && !isReviewReturn) {
          navigate(`/review/${currentInviteCode}`, { replace: true });
          return;
        }

        const participantsResponse = await listParticipants(gathering.id);
        if (!ignore) {
          setParticipants(participantsResponse.participants);
        }

        const cookieUser = getCookieUser();
        if (cookieUser) {
          setCurrentUser(cookieUser);
          setDisplayName(cookieUser);
        }

        const menuResponse = await listMenuItems(gathering.id);
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
  }, [currentUser, inviteCode, isReviewReturn, navigate]);

  const ownerOptions = useMemo(() => {
    const names = new Set(
      participants
        .filter((participant) => participant.display_name !== 'suite-admin')
        .map((participant) => participant.display_name)
        .filter(Boolean),
    );

    if (currentUser && currentUser !== 'suite-admin') {
      names.add(currentUser);
    }

    if (editingItem?.owner_name && editingItem.owner_name !== 'suite-admin') {
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
      const participant = response.participant;
      if (!participant) {
        throw new Error('System admin does not join menus as a participant.');
      }
      localStorage.setItem(
        participantStorageKey(inviteCode),
        participant.id,
      );
      localStorage.setItem(
        `letsorder:${inviteCode}:access_token`,
        response.access_token,
      );
      setCookieUser(name);
      setCurrentUser(name);
      setParticipantId(participant.id);
      setParticipants((items) => [participant, ...items]);
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
    setSelectedChef(currentUser);
    setSaveError(null);
    setIsEditorOpen(true);
  }

  function openEditDish(item: MenuItem) {
    if (isCurrentMenuLocked || (gatheringId && !participantId)) {
      return;
    }

    setEditingItem(item);
    setSelectedChef(item.owner_name ?? currentUser);
    setSaveError(null);
    setIsEditorOpen(true);
  }

  function closeEditor() {
    setIsEditorOpen(false);
    setEditingItem(null);
    setConflict(null);
  }

  function fillInput(name: string, value: string | number | null | undefined) {
    const field = dishEditorFormRef.current?.elements.namedItem(name);
    if (
      field instanceof HTMLInputElement ||
      field instanceof HTMLSelectElement ||
      field instanceof HTMLTextAreaElement
    ) {
      field.value = String(value ?? '');
    }
  }

  function applyRecommendation(item: MenuItem) {
    fillInput('name', item.name);
    fillInput('category', item.category ?? 'Main');
    fillInput('quantity', item.quantity);
    fillInput('unit', item.unit ?? 'plates');
    fillInput('reference_url', item.reference_url ?? '');
    fillInput('note', item.note ?? '');
    fillInput('status', 'planned');
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
          const pendingPayload = buildUpdatePayload(
            formData,
            participantId,
            currentUser,
            editingItem.revision,
          );
          const response = await updateMenuItem(editingItem.id, pendingPayload);

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
            reference_url: normalizeReferenceUrl(formData.get('reference_url')),
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
      if (error instanceof ApiError && error.status === 409) {
        const body = error.body as { latest_menu_item?: MenuItem } | null;
        if (editingItem && body?.latest_menu_item && participantId) {
          setConflict({
            latestItem: body.latest_menu_item,
            pendingPayload: buildUpdatePayload(
              formData,
              participantId,
              currentUser,
              body.latest_menu_item.revision,
            ),
          });
          setMenuItems((items) =>
            items.map((item) =>
              item.id === body.latest_menu_item?.id
                ? body.latest_menu_item
                : item,
            ),
          );
          return;
        }
      }

      setSaveError(
        error instanceof Error ? error.message : 'Failed to save menu item.',
      );
    } finally {
      setIsSaving(false);
    }
  }

  function useLatestConflictItem() {
    if (!conflict) {
      return;
    }

    setEditingItem(conflict.latestItem);
    setSelectedChef(conflict.latestItem.owner_name ?? currentUser);
    setConflict(null);
    setSaveError('Loaded the latest dish. Review it, then save again if needed.');
  }

  async function forceSaveConflictItem() {
    if (!conflict || !editingItem) {
      return;
    }

    setIsSaving(true);
    setSaveError(null);

    try {
      const response = await updateMenuItem(editingItem.id, conflict.pendingPayload);
      setMenuItems((items) =>
        items.map((item) =>
          item.id === response.menu_item.id ? response.menu_item : item,
        ),
      );
      setConflict(null);
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
            <p className="eyebrow">Menu workspace · {currentInviteCode}</p>
            <h1 className="menu-title">{currentTitle}</h1>
            <p className="lead">
              Add dishes, claim prep work, and keep the family menu tidy before
              it locks.
            </p>
          </div>
          {!needsDisplayName ? (
            <button
              disabled={isCurrentMenuLocked}
              type="button"
              onClick={openAddDish}
            >
              {isCurrentMenuLocked ? 'Menu locked' : 'Add dish'}
            </button>
          ) : null}
        </div>

        {!needsDisplayName ? (
          <>
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

            <div className="toolbar">
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
                  <option value="done">Done</option>
                  <option value="cancelled">Cancelled</option>
                </select>
              </label>
              <label className="status-filter">
                Category
                <select
                  value={categoryFilter}
                  onChange={(event) => setCategoryFilter(event.target.value)}
                >
                  <option value="all">All</option>
                  {categoryFilterOptions.map((category) => (
                    <option key={category} value={category}>
                      {category}
                    </option>
                  ))}
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
          </>
        ) : null}
      </section>

      {needsDisplayName ? (
        <div className="modal-overlay" role="presentation">
          <form
            aria-modal="true"
            aria-labelledby="join-menu-title"
            className="confirm-modal join-menu-modal"
            role="dialog"
            onSubmit={handleJoinMenu}
          >
            <div>
              <p className="card-kicker">Join menu</p>
              <h2 id="join-menu-title">Tell us who is editing</h2>
              <p>
                Enter your name before viewing or changing this menu, so the host
                can see who changed what.
              </p>
            </div>
            <label>
              Your display name
              <input
                required
                autoFocus
                minLength={1}
                pattern=".*\S.*"
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
        </div>
      ) : null}

      {isEditorOpen ? (
        <div className="modal-overlay" role="presentation" onClick={closeEditor}>
          <form
            ref={dishEditorFormRef}
            aria-modal="true"
            className="dish-editor-modal dish-editor-with-recommendations"
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
                  placeholder="1"
                  defaultValue={editingItem?.quantity ?? 1}
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
                <option value="done">Done</option>
                <option value="cancelled">Cancelled</option>
              </select>
            </label>

            <label>
              Chef
              <select
                key={`owner-${editingItem?.id ?? 'new'}`}
                name="owner_name"
                value={selectedChef}
                onChange={(event) => setSelectedChef(event.target.value)}
              >
                {ownerOptions.map((owner) => (
                  <option key={owner} value={owner}>
                    {owner}
                  </option>
                ))}
              </select>
            </label>

            <label>
              Reference link
              <input
                key={`reference-${editingItem?.id ?? 'new'}`}
                name="reference_url"
                placeholder="Paste a link or share text"
                defaultValue={editingItem?.reference_url ?? ''}
              />
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

            <aside className="dish-recommendations">
              <p className="card-kicker">Recommend</p>
              <h3>{selectedChef ? `${selectedChef}'s dishes` : 'Choose a Chef'}</h3>
              {chefRecommendations.length > 0 ? (
                <div className="recommendation-list">
                  {chefRecommendations.map((item) => (
                    <button
                      className="recommendation-card"
                      key={item.id}
                      type="button"
                      onClick={() => applyRecommendation(item)}
                    >
                      <strong>{item.name}</strong>
                      <span>
                        {item.category ?? 'Other'} · {item.quantity} {item.unit}
                      </span>
                    </button>
                  ))}
                </div>
              ) : (
                <p className="empty-panel-note">
                  No previous dishes for this Chef yet.
                </p>
              )}
            </aside>
          </form>
        </div>
      ) : null}

      {conflict ? (
        <div className="modal-overlay" role="presentation">
          <section
            aria-modal="true"
            aria-labelledby="conflict-title"
            className="confirm-modal conflict-modal"
            role="dialog"
          >
            <div>
              <p className="card-kicker">Conflict detected</p>
              <h2 id="conflict-title">This dish changed while you were editing</h2>
              <p>
                Someone saved a newer version of this dish. Choose whether to
                review the latest version or overwrite it with your changes.
              </p>
            </div>
            <div className="conflict-comparison">
              <div>
                <strong>Latest</strong>
                <span>{conflict.latestItem.name}</span>
                <span>
                  {conflict.latestItem.quantity} {conflict.latestItem.unit}
                </span>
                <span>{conflict.latestItem.status}</span>
              </div>
              <div>
                <strong>Your Change</strong>
                <span>{conflict.pendingPayload.name}</span>
                <span>
                  {conflict.pendingPayload.quantity} {conflict.pendingPayload.unit}
                </span>
                <span>{conflict.pendingPayload.status}</span>
              </div>
            </div>
            <div className="action-row modal-actions">
              <button type="button" onClick={useLatestConflictItem}>
                Use latest
              </button>
              <button
                className="danger-button"
                disabled={isSaving}
                type="button"
                onClick={forceSaveConflictItem}
              >
                {isSaving ? 'Saving...' : 'Use mine'}
              </button>
              <button
                className="ghost-button"
                type="button"
                onClick={() => setConflict(null)}
              >
                Cancel
              </button>
            </div>
          </section>
        </div>
      ) : null}
    </div>
  );
}
