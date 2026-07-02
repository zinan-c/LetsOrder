export type MenuItemStatus = 'planned' | 'prepared' | 'cancelled';

export interface MenuItem {
  id: string;
  gathering_id: string;
  created_by: string;
  updated_by?: string | null;
  name: string;
  category?: string | null;
  quantity: number;
  unit?: string | null;
  owner_name?: string | null;
  note?: string | null;
  status: MenuItemStatus;
  created_at: string;
  updated_at: string;
}
