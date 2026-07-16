import { apiRequest } from './client';
import type {
  DishRecommendation,
  MenuItem,
  MenuItemRatingSummary,
  MenuItemStatus,
} from '../types/menu';

export interface MenuItemsResponse {
  menu_items: MenuItem[];
}

export interface MenuItemResponse {
  menu_item: MenuItem;
}

export interface MenuRatingsResponse {
  ratings: MenuItemRatingSummary[];
}

export interface MenuRatingResponse {
  rating: MenuItemRatingSummary;
}

export interface DishRecommendationsResponse {
  recommendations: DishRecommendation[];
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
  expected_revision?: number;
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

export function listMenuRatings(gatheringId: string) {
  return apiRequest<MenuRatingsResponse>(`/api/gatherings/${gatheringId}/menu-ratings`);
}

export function listDishRecommendations(chefName: string, limit = 8) {
  const params = new URLSearchParams({ limit: String(limit) });

  return apiRequest<DishRecommendationsResponse>(
    `/api/chefs/${encodeURIComponent(chefName)}/dish-recommendations?${params}`,
  );
}

export function rateMenuItem(menuItemId: string, rating: number) {
  return apiRequest<MenuRatingResponse>(`/api/menu-items/${menuItemId}/rating`, {
    method: 'POST',
    body: JSON.stringify({ rating }),
  });
}
