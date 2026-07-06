import type { MenuItem } from '../types/menu';

export const mockGathering = {
  title: 'Saturday Hotpot Night',
  description:
    'A cozy family dinner where everyone can add dishes, prep notes, and little cravings before the menu locks.',
  inviteCode: 'hotpot-8f3a',
  status: 'active',
  hostName: 'Aunt May',
  expiresAt: '2026-07-03T18:00:00+08:00',
  participantCount: 7,
};

export const mockParticipants = [
  { id: 'p1', name: 'Aunt May', role: 'Host', joinedAt: '09:10' },
  { id: 'p2', name: 'Nico', role: 'Participant', joinedAt: '09:18' },
  { id: 'p3', name: 'Grandma Lin', role: 'Participant', joinedAt: '09:31' },
  { id: 'p4', name: 'Mia', role: 'Participant', joinedAt: '10:02' },
];

export const mockMenuItems: MenuItem[] = [
  {
    id: 'm1',
    gathering_id: 'g1',
    created_by: 'p1',
    updated_by: 'p2',
    name: 'Tomato beef hotpot base',
    category: 'Main',
    quantity: 1,
    unit: 'pot',
    owner_name: 'Aunt May',
    note: 'Make it mild first; chili oil on the side.',
    status: 'planned',
    created_at: '2026-07-02T09:20:00+08:00',
    updated_at: '2026-07-02T09:40:00+08:00',
  },
  {
    id: 'm2',
    gathering_id: 'g1',
    created_by: 'p2',
    updated_by: null,
    name: 'Handmade fish balls',
    category: 'Protein',
    quantity: 2,
    unit: 'boxes',
    owner_name: 'Nico',
    note: 'Buy from the market near the station.',
    status: 'done',
    created_at: '2026-07-02T10:12:00+08:00',
    updated_at: '2026-07-02T10:12:00+08:00',
  },
  {
    id: 'm3',
    gathering_id: 'g1',
    created_by: 'p3',
    updated_by: null,
    name: 'Lotus root and mushrooms',
    category: 'Vegetables',
    quantity: 3,
    unit: 'plates',
    owner_name: 'Grandma Lin',
    note: 'Slice thinly before dinner.',
    status: 'planned',
    created_at: '2026-07-02T10:30:00+08:00',
    updated_at: '2026-07-02T10:30:00+08:00',
  },
  {
    id: 'm4',
    gathering_id: 'g1',
    created_by: 'p4',
    updated_by: null,
    name: 'Mango sago',
    category: 'Dessert',
    quantity: 8,
    unit: 'cups',
    owner_name: 'Mia',
    note: 'Chill for at least two hours.',
    status: 'planned',
    created_at: '2026-07-02T11:01:00+08:00',
    updated_at: '2026-07-02T11:01:00+08:00',
  },
  {
    id: 'm5',
    gathering_id: 'g1',
    created_by: 'p2',
    updated_by: 'p2',
    name: 'Extra spicy broth',
    category: 'Main',
    quantity: 1,
    unit: 'pot',
    owner_name: 'Nico',
    note: 'Cancelled because not everyone can eat spicy food.',
    status: 'cancelled',
    created_at: '2026-07-02T11:20:00+08:00',
    updated_at: '2026-07-02T11:26:00+08:00',
  },
];

export const mockActivityLogs = [
  'Nico added handmade fish balls',
  'Grandma Lin updated lotus root quantity',
  'Mia added mango sago',
  'Nico marked extra spicy broth as cancelled',
];

export const mockPhotos = [
  {
    id: 'ph1',
    title: 'The table before everyone sat down',
    color: 'coral',
  },
  {
    id: 'ph2',
    title: 'Grandma guarding the soup like a legend',
    color: 'gold',
  },
  {
    id: 'ph3',
    title: 'Dessert survived for six whole minutes',
    color: 'mint',
  },
];
