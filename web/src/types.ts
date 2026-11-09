/** One account and, once they have one, their crew (from /juggle-crews/v1/me). */
export type Me = {
  uid: string;
  email: string;
  name: string;
  crew: Crew | null;
};
export type Crew = {
  id: string;
  name: string;
  timezone: string;
  invite_code: string;
  role: 'owner' | 'member';
  members: { uid: string; display_name: string; role: 'owner' | 'member' }[];
};
/** Routines-side session details (from /juggle-routines/v1/me). */
export type RoutinesMe = {
  uid: string;
  timezone: string;
  today: string;
  vapid_public_key: string;
};
export type Person = {
  id: string;
  name: string;
  tone: string;
  /** Set for people who can sign in; Mochi the cat has none. */
  account?: string | null;
  revision: number;
  created_date: string;
};
export type Routine = {
  id: string;
  revision: number;
  /** The assignee's name on that day; history keeps it. */
  assignee: string;
  person_id: string;
  name: string;
  note: string;
  group: string;
  time: string;
  /** ISO weekdays, Monday = 1. Empty means every day. */
  days: number[];
  start_date: string;
  end_date: string | null;
  enabled: boolean;
  prerequisite: string | null;
};
export type Version = { effective_from: string; routine: Routine };
export type Occurrence = {
  id: string;
  routine: Routine;
  revision: number;
  status: 'pending' | 'done' | 'cancelled';
  completed_by: string | null;
  completed_at: string | null;
};
export type Day = VisualMetadata & {
  date: string;
  today: string;
  timezone: string;
  items: Occurrence[];
  people: Person[];
  server_now: string;
  first_date: string;
  scheduled_times: Record<string, string>;
};
export type Schedule = VisualMetadata & {
  versions: Version[];
  people: Person[];
  today: string;
  timezone: string;
  scheduled_times: Record<string, string>;
};
export type Preferences = { timezone: string; revision: number };
export type CrewPreferences = {
  preferences: Preferences;
  members: { uid: string; name: string; preferences: Preferences }[];
};
export type Mood = 'happy' | 'neutral' | 'grumpy';
export type Appearance = { mood: Mood; revision: number };
export type AvatarMeta = { revision: number; has_photo: boolean };
export type VisualMetadata = {
  appearances?: Record<string, Appearance>;
  avatars?: Record<string, AvatarMeta>;
};
