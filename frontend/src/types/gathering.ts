export type GatheringStatus = 'draft' | 'active' | 'locked' | 'archived';

export interface Gathering {
  id: string;
  title: string;
  description?: string | null;
  invite_code: string;
  status: GatheringStatus;
  is_locked: boolean;
  starts_at?: string | null;
  expires_at: string;
  locked_at?: string | null;
  archived_at?: string | null;
  created_at: string;
  updated_at: string;
}

export interface GatheringListItem {
  id: string;
  title: string;
  description?: string | null;
  invite_code: string;
  status: GatheringStatus;
  is_locked: boolean;
  expires_at: string;
  item_count: number;
  prepared_count: number;
  participant_count: number;
  created_at: string;
  updated_at: string;
}

export interface CreateGatheringRequest {
  title: string;
  description?: string;
  host_name: string;
  starts_at?: string;
  expires_at: string;
}

export interface Participant {
  id: string;
  gathering_id: string;
  user_id?: string | null;
  display_name: string;
  role: string;
  last_menu_activity_at?: string | null;
  joined_at: string;
  created_at: string;
  updated_at: string;
}

export interface ActivityLog {
  id: string;
  gathering_id: string;
  actor_id?: string | null;
  actor_name?: string | null;
  action: string;
  target_type: string;
  target_id?: string | null;
  detail?: string | null;
  created_at: string;
}

export interface Photo {
  id: string;
  gathering_id: string;
  uploaded_by: string;
  file_url: string;
  thumbnail_url?: string | null;
  caption?: string | null;
  taken_at?: string | null;
  created_at: string;
  updated_at: string;
}
