export type MenuItemStatus = 'planned' | 'prepared' | 'done' | 'cancelled';

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
  reference_url?: string | null;
  note?: string | null;
  status: MenuItemStatus;
  revision: number;
  created_at: string;
  updated_at: string;
}

export interface MenuItemRatingSummary {
  menu_item_id: string;
  average_rating?: number | null;
  rating_count: number;
  my_rating?: number | null;
}
