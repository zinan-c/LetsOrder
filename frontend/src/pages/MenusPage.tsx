import { useEffect, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import {
  deleteGathering,
  listGatherings,
} from '../api/gatherings';
import PageCard from '../components/PageCard';
import StatusPill from '../components/StatusPill';
import type { GatheringListItem } from '../types/gathering';
import { getCurrentUser, USER_CHANGED_EVENT } from '../utils/user';

export default function MenusPage() {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const [currentUser, setCurrentUser] = useState(() => getCurrentUser());
  const isAdmin = currentUser?.role === 'admin';
  const [menuToDelete, setMenuToDelete] = useState<GatheringListItem | null>(
    null,
  );
  const gatheringsQuery = useQuery({
    queryKey: ['gatherings', currentUser?.id],
    queryFn: listGatherings,
    enabled: Boolean(currentUser),
    retry: false,
  });
  const menus = gatheringsQuery.data?.gatherings ?? [];
  const deleteMutation = useMutation({
    mutationFn: deleteGathering,
    onSuccess: async () => {
      setMenuToDelete(null);
      await queryClient.invalidateQueries({ queryKey: ['gatherings'] });
    },
  });

  useEffect(() => {
    function handleUserChanged(event: Event) {
      const user = event instanceof CustomEvent ? event.detail : null;
      setCurrentUser(user);
    }

    window.addEventListener(USER_CHANGED_EVENT, handleUserChanged);

    return () => {
      window.removeEventListener(USER_CHANGED_EVENT, handleUserChanged);
    };
  }, []);

  return (
    <PageCard
      eyebrow="Menus"
      title="Choose one menu"
      description="Browse family gathering menus, then open one to edit dishes, claim prep work, and review what changed."
    >
      <div className="menu-list">
        {menus.map((menu) => (
          <article
            className="menu-list-row"
            key={menu.id}
            role="link"
            tabIndex={0}
            onClick={() => navigate(`/menu/${menu.invite_code}`)}
            onKeyDown={(event) => {
              if (event.key === 'Enter' || event.key === ' ') {
                event.preventDefault();
                navigate(`/menu/${menu.invite_code}`);
              }
            }}
          >
            <div>
              <h2>{menu.title}</h2>
              <p>{menu.description}</p>
            </div>
            <div className="menu-list-meta">
              <StatusPill>{menu.status}</StatusPill>
              <strong>{menu.item_count} items</strong>
              <span>{menu.prepared_count} prepared</span>
              <span>{menu.participant_count} people</span>
            </div>
            {isAdmin ? (
              <div className="menu-list-actions">
                <button
                  className="danger-button"
                  disabled={deleteMutation.isPending}
                  type="button"
                  onClick={(event) => {
                    event.stopPropagation();
                    setMenuToDelete(menu);
                  }}
                >
                  Delete
                </button>
              </div>
            ) : null}
          </article>
        ))}
      </div>
      {gatheringsQuery.isLoading ? (
        <p className="empty-panel-note">Loading menus...</p>
      ) : null}
      {gatheringsQuery.isError ? (
        <p className="error">Could not load menus. Please try again.</p>
      ) : null}
      {!gatheringsQuery.isLoading && !gatheringsQuery.isError && menus.length === 0 ? (
        <p className="empty-panel-note">
          {isAdmin ? 'No gatherings yet.' : 'No menus joined yet.'}
        </p>
      ) : null}

      {menuToDelete ? (
        <div
          className="modal-overlay"
          role="presentation"
          onClick={() => setMenuToDelete(null)}
        >
          <section
            aria-modal="true"
            aria-labelledby="delete-menu-title"
            className="confirm-modal"
            role="dialog"
            onClick={(event) => event.stopPropagation()}
          >
            <h2 id="delete-menu-title">Delete Menu "{menuToDelete.title}"?</h2>
            <p>
              This removes the menu from the active list. The data is archived
              instead of permanently destroyed.
            </p>
            <div className="action-row modal-actions">
              <button
                className="danger-button"
                disabled={deleteMutation.isPending}
                type="button"
                onClick={() => deleteMutation.mutate(menuToDelete.id)}
              >
                {deleteMutation.isPending ? 'Deleting...' : 'Yes, delete'}
              </button>
              <button
                className="ghost-button"
                disabled={deleteMutation.isPending}
                type="button"
                onClick={() => setMenuToDelete(null)}
              >
                Cancel
              </button>
            </div>
          </section>
        </div>
      ) : null}
    </PageCard>
  );
}
