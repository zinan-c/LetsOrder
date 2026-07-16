import type { FormEvent, RefObject } from 'react';
import type { DishRecommendation, MenuItem } from '../types/menu';

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

interface DishEditorModalProps {
  chefRecommendations: DishRecommendation[];
  editingItem: MenuItem | null;
  formRef: RefObject<HTMLFormElement | null>;
  isLoadingRecommendations: boolean;
  isSaving: boolean;
  ownerOptions: string[];
  saveError: string | null;
  selectedChef: string;
  onApplyRecommendation: (item: DishRecommendation) => void;
  onClose: () => void;
  onSelectedChefChange: (value: string) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
}

export default function DishEditorModal({
  chefRecommendations,
  editingItem,
  formRef,
  isLoadingRecommendations,
  isSaving,
  ownerOptions,
  saveError,
  selectedChef,
  onApplyRecommendation,
  onClose,
  onSelectedChefChange,
  onSubmit,
}: DishEditorModalProps) {
  const isEditing = Boolean(editingItem);
  const editorTitle = isEditing ? 'Update this dish' : 'Add a new dish';

  return (
    <div className="modal-overlay" role="presentation" onClick={onClose}>
      <form
        ref={formRef}
        aria-modal="true"
        className="dish-editor-modal dish-editor-with-recommendations"
        role="dialog"
        aria-labelledby="dish-editor-title"
        onClick={(event) => event.stopPropagation()}
        onSubmit={onSubmit}
      >
        <div className="panel-header">
          <div>
            <p className="card-kicker">Dish editor</p>
            <h2 id="dish-editor-title">{editorTitle}</h2>
          </div>
          <button className="icon-button" type="button" onClick={onClose}>
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
            onChange={(event) => onSelectedChefChange(event.target.value)}
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
            {isSaving ? 'Saving...' : isEditing ? 'Save changes' : 'Add to menu'}
          </button>
          <button className="ghost-button" type="button" onClick={onClose}>
            Cancel
          </button>
        </div>

        <aside className="dish-recommendations">
          <p className="card-kicker">Recommend</p>
          <h3>{selectedChef ? `${selectedChef}'s dishes` : 'Choose a Chef'}</h3>
          {isLoadingRecommendations ? (
            <p className="empty-panel-note">Loading recommendations...</p>
          ) : chefRecommendations.length > 0 ? (
            <div className="recommendation-list">
              {chefRecommendations.map((item) => (
                <button
                  className="recommendation-card"
                  key={item.dish_key}
                  type="button"
                  onClick={() => onApplyRecommendation(item)}
                >
                  <strong>{item.name}</strong>
                  <span>
                    {item.category ?? 'Other'} · {item.quantity} {item.unit}
                  </span>
                  <span className="recommendation-rating">
                    {item.average_rating
                      ? `★ ${item.average_rating.toFixed(1)} · ${item.rating_count} rating${
                          item.rating_count === 1 ? '' : 's'
                        }`
                      : 'No rating yet'}
                  </span>
                </button>
              ))}
            </div>
          ) : (
            <p className="empty-panel-note">No previous dishes for this Chef yet.</p>
          )}
        </aside>
      </form>
    </div>
  );
}
