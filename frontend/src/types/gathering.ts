export type GatheringStatus = 'draft' | 'active' | 'locked' | 'archived';

export interface Gathering {
  id: string;
  title: string;
  description?: string | null;
  invite_code: string;
  status: GatheringStatus;
  starts_at?: string | null;
  expires_at: string;
  locked_at?: string | null;
  archived_at?: string | null;
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
