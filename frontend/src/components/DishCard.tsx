import type { MenuItem } from '../types/menu';
import StatusPill from './StatusPill';

const toneByStatus = {
  planned: 'warm',
  prepared: 'green',
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
      <div className="dish-card-header">
        <p className="card-kicker">{item.category}</p>
        <h3>{item.name}</h3>
      </div>
      <p className="dish-meta">
        {item.quantity} {item.unit}
      </p>
      <p className="dish-owner">
        {item.owner_name ? `Owner: ${item.owner_name}` : 'Owner: Unassigned'}
      </p>
      <p className="dish-note">{item.note}</p>
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
