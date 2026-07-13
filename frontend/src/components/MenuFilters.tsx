import type { MenuItemStatus } from '../types/menu';
import { formatDateTime } from '../utils/dateTime';

interface MenuFiltersProps {
  categoryFilter: string;
  categoryOptions: string[];
  expiresAt: string;
  isLocked: boolean;
  statusFilter: 'all' | MenuItemStatus;
  onCategoryFilterChange: (value: string) => void;
  onStatusFilterChange: (value: 'all' | MenuItemStatus) => void;
}

export default function MenuFilters({
  categoryFilter,
  categoryOptions,
  expiresAt,
  isLocked,
  statusFilter,
  onCategoryFilterChange,
  onStatusFilterChange,
}: MenuFiltersProps) {
  return (
    <div className="toolbar">
      <label className="status-filter">
        Status
        <select
          value={statusFilter}
          onChange={(event) =>
            onStatusFilterChange(event.target.value as 'all' | MenuItemStatus)
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
          onChange={(event) => onCategoryFilterChange(event.target.value)}
        >
          <option value="all">All</option>
          {categoryOptions.map((category) => (
            <option key={category} value={category}>
              {category}
            </option>
          ))}
        </select>
      </label>
      <span className="toolbar-note">
        {isLocked ? 'Menu editing is locked' : `Menu locks ${formatDateTime(expiresAt)}`}
      </span>
    </div>
  );
}
