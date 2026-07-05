import { apiRequest } from './client';
import type { MenuItem, MenuItemStatus } from '../types/menu';

export interface MenuItemsResponse {
  menu_items: MenuItem[];
}

export interface MenuItemResponse {
  menu_item: MenuItem;
}

export interface CreateMenuItemPayload {
  created_by: string;
  name: string;
  category?: string;
  quantity?: number;
  unit?: string;
  owner_name?: string;
  reference_url?: string;
  note?: string;
  status?: MenuItemStatus;
}

export interface UpdateMenuItemPayload {
  updated_by: string;
  name?: string;
  category?: string;
  quantity?: number;
  unit?: string;
  owner_name?: string;
  reference_url?: string;
  note?: string;
  status?: MenuItemStatus;
}

export function listMenuItems(gatheringId: string) {
  return apiRequest<MenuItemsResponse>(`/api/gatherings/${gatheringId}/menu-items`);
}

export function createMenuItem(
  gatheringId: string,
  payload: CreateMenuItemPayload,
) {
  return apiRequest<MenuItemResponse>(`/api/gatherings/${gatheringId}/menu-items`, {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export function updateMenuItem(menuItemId: string, payload: UpdateMenuItemPayload) {
  return apiRequest<MenuItemResponse>(`/api/menu-items/${menuItemId}`, {
    method: 'PATCH',
    body: JSON.stringify(payload),
  });
}
