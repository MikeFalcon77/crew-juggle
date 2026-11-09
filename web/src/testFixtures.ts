// Test-only fixtures: Flat 4B with Pip (you), Zed and Mochi the cat.
import type { Day, Me, Occurrence, Person, Routine, Schedule } from './types';

export const crewMe: Me = {
  uid: 'demo-pip',
  email: 'pip@demo.crew-juggle.test',
  name: 'Pip',
  crew: {
    id: 'crew-1',
    name: 'Flat 4B',
    timezone: 'Europe/Berlin',
    invite_code: 'K7QX-M2PA',
    role: 'owner',
    members: [
      { uid: 'demo-pip', display_name: 'Pip', role: 'owner' },
      { uid: 'demo-zed', display_name: 'Zed', role: 'member' },
    ],
  },
};

export const people: Person[] = [
  {
    id: 'p-pip',
    name: 'Pip',
    tone: 'blue',
    account: 'demo-pip',
    revision: 1,
    created_date: '2026-11-10',
  },
  {
    id: 'p-zed',
    name: 'Zed',
    tone: 'lilac',
    account: 'demo-zed',
    revision: 1,
    created_date: '2026-11-10',
  },
  {
    id: 'mochi',
    name: 'Mochi',
    tone: 'peach',
    account: null,
    revision: 1,
    created_date: '2026-11-10',
  },
];

export function routine(
  id: string,
  person: Person,
  name: string,
  time: string,
  extra: Partial<Routine> = {},
): Routine {
  return {
    id,
    revision: 1,
    assignee: person.name,
    person_id: person.id,
    name,
    note: '',
    group: '',
    time,
    days: [],
    start_date: '2026-11-10',
    end_date: null,
    enabled: true,
    prerequisite: null,
    ...extra,
  };
}

export function occurrence(r: Routine, extra: Partial<Occurrence> = {}): Occurrence {
  return {
    id: `2026-11-10/${r.id}`,
    routine: r,
    revision: 1,
    status: 'pending',
    completed_by: null,
    completed_at: null,
    ...extra,
  };
}

/** Tuesday in Flat 4B at 10:30 Berlin time. */
export function flatDay(): Day {
  const [pip, zed, mochi] = people;
  return {
    date: '2026-11-10',
    today: '2026-11-10',
    first_date: '2026-11-07',
    timezone: 'Europe/Berlin',
    server_now: '2026-11-10T09:30:00Z',
    scheduled_times: {
      '08:00': '2026-11-10T07:00:00Z',
      '09:00': '2026-11-10T08:00:00Z',
      '11:00': '2026-11-10T10:00:00Z',
      '19:30': '2026-11-10T18:30:00Z',
      '21:00': '2026-11-10T20:00:00Z',
    },
    people,
    items: [
      occurrence(
        routine('mochi-breakfast', pip, 'Breakfast for Mochi', '08:00', { group: 'Mochi' }),
        {
          status: 'done',
          revision: 2,
          completed_by: 'demo-pip',
          completed_at: '2026-11-10T06:55:00Z',
        },
      ),
      occurrence(
        routine('dishes', zed, 'Do the dishes', '09:00', { note: 'Robot parts are not dishes.' }),
      ),
      occurrence(routine('wash', zed, 'Start the laundry', '11:00', { group: 'Laundry' })),
      occurrence(
        routine('hang', zed, 'Hang the laundry', '11:00', {
          group: 'Laundry',
          prerequisite: 'wash',
        }),
      ),
      occurrence(routine('trash', pip, 'Take out the trash', '19:30', { days: [2] })),
      occurrence(routine('supervise', mochi, 'Supervise kitchen cleaning 😼', '21:00'), {
        status: 'done',
        revision: 2,
        completed_by: 'demo-zed',
        completed_at: '2026-11-10T09:00:00Z',
      }),
    ],
  };
}

export function flatSchedule(): Schedule {
  const day = flatDay();
  return {
    versions: day.items.map((i) => ({ effective_from: '2026-11-10', routine: i.routine })),
    people,
    today: '2026-11-10',
    timezone: 'Europe/Berlin',
    scheduled_times: {},
  };
}
