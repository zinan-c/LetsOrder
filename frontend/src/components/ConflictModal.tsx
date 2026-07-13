import type { UpdateMenuItemPayload } from '../api/menuItems';
import type { MenuItem } from '../types/menu';

interface ConflictModalProps {
  conflict: {
    latestItem: MenuItem;
    pendingPayload: UpdateMenuItemPayload;
  };
  isSaving: boolean;
  onCancel: () => void;
  onUseLatest: () => void;
  onUseMine: () => void;
}

export default function ConflictModal({
  conflict,
  isSaving,
  onCancel,
  onUseLatest,
  onUseMine,
}: ConflictModalProps) {
  return (
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
            Someone saved a newer version of this dish. Choose whether to review
            the latest version or overwrite it with your changes.
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
          <button type="button" onClick={onUseLatest}>
            Use latest
          </button>
          <button
            className="danger-button"
            disabled={isSaving}
            type="button"
            onClick={onUseMine}
          >
            {isSaving ? 'Saving...' : 'Use mine'}
          </button>
          <button className="ghost-button" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </section>
    </div>
  );
}
