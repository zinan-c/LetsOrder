import { FormEvent, useEffect, useMemo, useRef, useState } from 'react';
import { useLocation, useNavigate, useParams } from 'react-router-dom';
import {
  joinGathering,
  joinGatheringByInviteCode,
  listParticipants,
} from '../api/gatherings';
import {
  createMenuItem,
  listDishRecommendations,
  listMenuItems,
  updateMenuItem,
  type UpdateMenuItemPayload,
} from '../api/menuItems';
import { ApiError } from '../api/client';
import ConflictModal from '../components/ConflictModal';
import DishCard from '../components/DishCard';
import DishEditorModal from '../components/DishEditorModal';
import GatheringSummary from '../components/GatheringSummary';
import JoinMenuModal from '../components/JoinMenuModal';
import MenuFilters from '../components/MenuFilters';
import type { Gathering, Participant } from '../types/gathering';
import type { DishRecommendation, MenuItem, MenuItemStatus } from '../types/menu';
import {
  getCookieUser,
  getCurrentUser,
  setCookieUser,
  USER_CHANGED_EVENT,
} from '../utils/user';

function participantStorageKey(inviteCode?: string) {
  return `letsorder:${inviteCode ?? 'unknown'}:participant_id`;
}

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
  const [chefRecommendations, setChefRecommendations] = useState<DishRecommendation[]>([]);
  const [isLoadingRecommendations, setIsLoadingRecommendations] = useState(false);
  const [menuItems, setMenuItems] = useState<MenuItem[]>([]);
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
  const [loadError, setLoadError] = useState<string | null>(null);
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
        setLoadError(null);
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
          setMenuItems([]);
          setLoadError('Could not load this menu. Please check your access or try again.');
        }
      }
    }

    loadMenuItems();

    return () => {
      ignore = true;
    };
  }, [currentUser, inviteCode, isReviewReturn, navigate]);

  useEffect(() => {
    const chefName = selectedChef.trim();
    if (!isEditorOpen || !chefName) {
      setChefRecommendations([]);
      setIsLoadingRecommendations(false);
      return;
    }

    let ignore = false;

    async function loadDishRecommendations() {
      setIsLoadingRecommendations(true);

      try {
        const response = await listDishRecommendations(chefName);
        if (!ignore) {
          setChefRecommendations(response.recommendations);
        }
      } catch {
        if (!ignore) {
          setChefRecommendations([]);
        }
      } finally {
        if (!ignore) {
          setIsLoadingRecommendations(false);
        }
      }
    }

    loadDishRecommendations();

    return () => {
      ignore = true;
    };
  }, [isEditorOpen, selectedChef]);

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

  function applyRecommendation(item: DishRecommendation) {
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
      if (!canUseApi || !gatheringId || !participantId) {
        throw new Error('You need to join this menu before editing dishes.');
      }

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

  const currentTitle = currentGathering?.title ?? 'Menu unavailable';
  const currentDescription = currentGathering?.description ?? '';
  const currentExpiresAt = currentGathering?.expires_at ?? new Date().toISOString();
  const currentInviteCode =
    currentGathering?.invite_code ?? inviteCode ?? 'unknown';
  const isCurrentMenuLocked = currentGathering?.is_locked ?? false;
  const isAdmin = getCurrentUser()?.role === 'admin';
  const needsDisplayName = Boolean(currentGathering && !participantId && !isAdmin);
  const canEditMenu = Boolean(participantId) && !isCurrentMenuLocked;
  const adminAddDishHint = isAdmin && !participantId && !isCurrentMenuLocked;

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
            <div className="add-dish-action">
              <button
                disabled={!canEditMenu}
                type="button"
                onClick={openAddDish}
              >
                {isCurrentMenuLocked ? 'Menu locked' : 'Add dish'}
              </button>
              {adminAddDishHint ? (
                <p className="action-hint">
                  Admin can manage this gathering, but dishes must be edited by
                  participants.
                </p>
              ) : null}
            </div>
          ) : null}
        </div>

        {loadError ? <p className="error">{loadError}</p> : null}

        {!needsDisplayName ? (
          <>
            <GatheringSummary
              title={currentTitle}
              description={currentDescription}
              inviteCode={currentInviteCode}
              expiresAt={currentExpiresAt}
              isLocked={isCurrentMenuLocked}
              participantCount={undefined}
            />

            <MenuFilters
              categoryFilter={categoryFilter}
              categoryOptions={categoryFilterOptions}
              expiresAt={currentExpiresAt}
              isLocked={isCurrentMenuLocked}
              statusFilter={statusFilter}
              onCategoryFilterChange={setCategoryFilter}
              onStatusFilterChange={setStatusFilter}
            />

            <div className="dish-list">
              {sortedMenuItems.map((item) => (
                <DishCard
                  item={item}
                  key={item.id}
                  readOnly={!canEditMenu}
                  onEdit={openEditDish}
                />
              ))}
            </div>
          </>
        ) : null}
      </section>

      {needsDisplayName ? (
        <JoinMenuModal
          displayName={displayName}
          error={joinError}
          isJoining={isJoining}
          onDisplayNameChange={setDisplayName}
          onSubmit={handleJoinMenu}
        />
      ) : null}

      {isEditorOpen ? (
        <DishEditorModal
          chefRecommendations={chefRecommendations}
          editingItem={editingItem}
          formRef={dishEditorFormRef}
          isLoadingRecommendations={isLoadingRecommendations}
          isSaving={isSaving}
          ownerOptions={ownerOptions}
          saveError={saveError}
          selectedChef={selectedChef}
          onApplyRecommendation={applyRecommendation}
          onClose={closeEditor}
          onSelectedChefChange={setSelectedChef}
          onSubmit={handleSaveMenuItem}
        />
      ) : null}

      {conflict ? (
        <ConflictModal
          conflict={conflict}
          isSaving={isSaving}
          onCancel={() => setConflict(null)}
          onUseLatest={useLatestConflictItem}
          onUseMine={forceSaveConflictItem}
        />
      ) : null}
    </div>
  );
}
