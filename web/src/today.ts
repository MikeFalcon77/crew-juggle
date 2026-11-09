import { canMark } from './DayView';
import type { Day, Occurrence } from './types';
import type { AppRoute } from './routes';
import { ListChecks, ShoppingBag, type LucideIcon } from 'lucide-react';
export type CardGroup = 'attention' | 'good' | 'neutral' | 'next';
export type TrackCard = {
  id: string;
  personId: string;
  personName?: string;
  track: 'routines' | 'shopping';
  group: CardGroup;
  title: string;
  detail: string;
  target: AppRoute;
  sort: string;
  action?: Occurrence;
};
/** What Today needs from the shopping gear. */
export type ShoppingGlance = {
  list: { name: string; done: boolean; author: string }[];
  receipts: { id: string; merchant: string; status: string; registeredBy: string; date: string }[];
};
export type TrackInput = { day?: Day; shopping?: ShoppingGlance; today: string };
export type TrackContribution = {
  id: 'routines' | 'shopping';
  label: string;
  target: AppRoute;
  Icon: LucideIcon;
  cards: (input: TrackInput) => TrackCard[];
};
function routineCards({ day, today }: TrackInput): TrackCard[] {
  if (!day || day.date !== today) return [];
  const clock = day.server_now;
  const cards: TrackCard[] = [];
  for (const person of day.people) {
    const items = day.items.filter(
      (i) => i.routine.person_id === person.id && i.status !== 'cancelled',
    );
    if (!items.length) continue;
    const pending = items
      .filter((i) => i.status === 'pending')
      .sort(
        (a, b) =>
          a.routine.time.localeCompare(b.routine.time) || a.routine.id.localeCompare(b.routine.id),
      );
    const due = pending.filter(
      (i) =>
        day.scheduled_times[i.routine.time] &&
        Date.parse(day.scheduled_times[i.routine.time]) <= Date.parse(clock),
    );
    const next = (due.length ? due : pending).find((i) => canMark(i, items));
    const done = items.length - pending.length;
    const group: CardGroup = !pending.length ? 'good' : due.length ? 'attention' : 'next';
    cards.push({
      id: `routines:${person.id}`,
      personId: person.id,
      track: 'routines',
      group,
      title: !pending.length
        ? 'All done for today'
        : due.length
          ? `${due.length} overdue`
          : 'Up next',
      detail: `${done} of ${items.length} done${next ? ` · ${next.routine.time} · ${next.routine.name}` : ''}`,
      target: {
        page: 'routines',
        tab: 'today',
        person: person.id,
        record: next ? `occurrence:${next.routine.id}` : undefined,
      },
      sort: next?.routine.time || '23:59',
      action: next,
    });
  }
  return cards;
}
function shoppingCards({ shopping, today }: TrackInput): TrackCard[] {
  if (!shopping) return [];
  const cards: TrackCard[] = [];
  const open = shopping.list.filter((i) => !i.done);
  if (open.length)
    cards.push({
      id: 'shopping:list',
      personId: '',
      personName: 'Shopping list',
      track: 'shopping',
      group: 'next',
      title: `${open.length} thing${open.length === 1 ? '' : 's'} to buy`,
      detail: open
        .slice(0, 4)
        .map((i) => i.name)
        .join(', ')
        .concat(open.length > 4 ? '…' : ''),
      target: { page: 'shopping', tab: 'list' },
      sort: '99:99',
    });
  for (const receipt of shopping.receipts.filter((r) => r.date === today)) {
    if (receipt.status === 'saved')
      cards.push({
        id: `shopping:receipt:${receipt.id}`,
        personId: '',
        personName: receipt.registeredBy,
        track: 'shopping',
        group: 'good',
        title: `Receipt from ${receipt.merchant}`,
        detail: `${receipt.registeredBy} scanned it · the list updated itself`,
        target: { page: 'shopping', tab: 'history' },
        sort: receipt.id,
      });
    if (receipt.status === 'review')
      cards.push({
        id: `shopping:review:${receipt.id}`,
        personId: '',
        personName: receipt.registeredBy,
        track: 'shopping',
        group: 'attention',
        title: 'A receipt needs a look',
        detail: `${receipt.merchant} · scanned by ${receipt.registeredBy}`,
        target: { page: 'shopping', tab: 'history' },
        sort: receipt.id,
      });
  }
  return cards;
}
export const tracks: TrackContribution[] = [
  {
    id: 'routines',
    label: 'Routines',
    target: { page: 'routines', tab: 'today' },
    Icon: ListChecks,
    cards: routineCards,
  },
  {
    id: 'shopping',
    label: 'Shopping',
    target: { page: 'shopping', tab: 'list' },
    Icon: ShoppingBag,
    cards: shoppingCards,
  },
];
export function todayCards(input: TrackInput) {
  return tracks.flatMap((t) => t.cards(input));
}
