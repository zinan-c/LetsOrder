import type { Participant } from './gathering';

export interface User {
  id: string;
  username: string;
  display_name: string;
  role: 'admin' | 'user';
  created_at: string;
  updated_at: string;
}

export interface AuthResponse {
  user: User;
  token: string;
  generated_password?: string | null;
  participant?: Participant | null;
}
