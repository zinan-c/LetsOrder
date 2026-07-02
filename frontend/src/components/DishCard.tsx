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
}

export default function DishCard({ item, readOnly = false }: DishCardProps) {
  return (
    <article className={`dish-card dish-card-${item.status}`}>
      <div className="dish-card-header">
        <div>
          <p className="card-kicker">{item.category}</p>
          <h3>{item.name}</h3>
        </div>
        <StatusPill tone={toneByStatus[item.status]}>{item.status}</StatusPill>
      </div>
      <p className="dish-meta">
        {item.quantity} {item.unit} · Owner: {item.owner_name}
      </p>
      <p className="dish-note">{item.note}</p>
      {readOnly ? null : (
        <button className="ghost-button" type="button">
          Edit item
        </button>
      )}
    </article>
  );
}
