import type { MenuItem } from '../types/menu';
import StatusPill from './StatusPill';

const toneByStatus = {
  planned: 'warm',
  prepared: 'green',
  done: 'neutral',
  cancelled: 'red',
} as const;

interface DishCardProps {
  item: MenuItem;
  readOnly?: boolean;
  onEdit?: (item: MenuItem) => void;
}

export default function DishCard({
  item,
  readOnly = false,
  onEdit,
}: DishCardProps) {
  return (
    <article className={`dish-card dish-card-${item.status}`}>
      <p className="dish-category">{item.category ?? 'Other'}</p>
      <h3>{item.name}</h3>
      <p className="dish-meta">
        {item.quantity} {item.unit}
      </p>
      <p className="dish-owner">
        {item.owner_name ? `Chef: ${item.owner_name}` : 'Chef: Unassigned'}
      </p>
      <div className="dish-reference">
        {item.reference_url ? (
          <a
            className="button-link secondary mini-link-button"
            href={item.reference_url}
            rel="noreferrer"
            target="_blank"
          >
            Ref Link
          </a>
        ) : (
          <span>—</span>
        )}
      </div>
      <StatusPill tone={toneByStatus[item.status]}>{item.status}</StatusPill>
      {readOnly ? null : (
        <button
          className="ghost-button"
          type="button"
          onClick={() => onEdit?.(item)}
        >
          Edit item
        </button>
      )}
    </article>
  );
}
