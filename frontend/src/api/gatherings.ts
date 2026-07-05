import { apiRequest } from './client';
import type {
  CreateGatheringRequest,
  ActivityLog,
  Gathering,
  GatheringListItem,
  Participant,
  Photo,
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

export interface ListParticipantsResponse {
  participants: Participant[];
}

export interface ListActivityLogsResponse {
  activity_logs: ActivityLog[];
}

export interface ListPhotosResponse {
  photos: Photo[];
}

export interface UploadPhotoResponse {
  photo: Photo;
}

export interface UpdatePhotoResponse {
  photo: Photo;
}

export interface JoinGatheringResponse {
  participant: Participant;
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

export function updateGatheringDeadline(gatheringId: string, expiresAt: string) {
  return apiRequest<GetGatheringResponse>(`/api/gatherings/${gatheringId}`, {
    method: 'PATCH',
    body: JSON.stringify({ expires_at: expiresAt }),
  });
}

export function lockGathering(gatheringId: string) {
  return apiRequest<GetGatheringResponse>(`/api/gatherings/${gatheringId}/lock`, {
    method: 'POST',
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

export function listParticipants(gatheringId: string) {
  return apiRequest<ListParticipantsResponse>(
    `/api/gatherings/${gatheringId}/participants`,
  );
}

export function listActivityLogs(gatheringId: string) {
  return apiRequest<ListActivityLogsResponse>(
    `/api/gatherings/${gatheringId}/activity-logs`,
  );
}

export function listPhotos(gatheringId: string) {
  return apiRequest<ListPhotosResponse>(`/api/gatherings/${gatheringId}/photos`);
}

export function uploadPhoto(gatheringId: string, file: File, caption?: string) {
  const formData = new FormData();
  formData.append('file', file);
  if (caption) {
    formData.append('caption', caption);
  }

  return apiRequest<UploadPhotoResponse>(`/api/gatherings/${gatheringId}/photos`, {
    method: 'POST',
    body: formData,
  });
}

export function updatePhotoCaption(photoId: string, caption: string) {
  return apiRequest<UpdatePhotoResponse>(`/api/photos/${photoId}`, {
    method: 'PATCH',
    body: JSON.stringify({ caption }),
  });
}

export function deletePhoto(photoId: string) {
  return apiRequest<UpdatePhotoResponse>(`/api/photos/${photoId}`, {
    method: 'DELETE',
  });
}
