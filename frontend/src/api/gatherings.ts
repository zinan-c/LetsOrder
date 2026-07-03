import { apiRequest } from './client';
import type {
  CreateGatheringRequest,
  Gathering,
  GatheringListItem,
} from '../types/gathering';

export interface CreateGatheringResponse {
  gathering: Gathering;
  host: {
    id: string;
    gathering_id: string;
    display_name: string;
    role: string;
  };
  access_token: string;
}

export interface GetGatheringResponse {
  gathering: Gathering;
}

export interface ListGatheringsResponse {
  gatherings: GatheringListItem[];
}

export interface JoinGatheringResponse {
  participant: {
    id: string;
    gathering_id: string;
    display_name: string;
    role: string;
  };
  access_token: string;
}

export function createGathering(payload: CreateGatheringRequest) {
  return apiRequest<CreateGatheringResponse>('/api/gatherings', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export function listGatherings() {
  return apiRequest<ListGatheringsResponse>('/api/gatherings');
}

export function getGatheringByInviteCode(inviteCode: string) {
  return apiRequest<GetGatheringResponse>(`/api/gatherings/${inviteCode}`);
}

export function deleteGathering(gatheringId: string) {
  return apiRequest<GetGatheringResponse>(`/api/gatherings/${gatheringId}`, {
    method: 'DELETE',
  });
}

export function joinGathering(gatheringId: string, displayName: string) {
  return apiRequest<JoinGatheringResponse>(
    `/api/gatherings/${gatheringId}/participants`,
    {
      method: 'POST',
      body: JSON.stringify({ display_name: displayName }),
    },
  );
}
