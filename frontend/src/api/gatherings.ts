import { apiRequest } from './client';
import type { CreateGatheringRequest, Gathering } from '../types/gathering';

export interface CreateGatheringResponse {
  gathering: Gathering;
}

export function createGathering(payload: CreateGatheringRequest) {
  return apiRequest<CreateGatheringResponse>('/api/gatherings', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}
