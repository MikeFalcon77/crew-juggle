import type { Day, Occurrence } from './types';
export type Celebration = {
  id: string;
  personId: string;
  itemId: string;
  message: string;
  level: 'item' | 'period' | 'day';
};
export function celebrationFor(day: Day, item: Occurrence, id: string): Celebration | null {
  const before = day.items.find((i) => i.id === item.id);
  if (!before || before.status !== 'pending' || item.status !== 'done') return null;
  const all = day.items.filter(
    (i) => i.status !== 'cancelled' && i.routine.person_id === item.routine.person_id,
  );
  const pending = (items: Occurrence[]) =>
    items.some((i) => i.id !== item.id && i.status !== 'done');
  const hour = Number(item.routine.time.slice(0, 2));
  const period = hour < 12 ? 0 : hour < 18 ? 1 : 2;
  const group = all.filter((i) => {
    const h = Number(i.routine.time.slice(0, 2));
    return (h < 12 ? 0 : h < 18 ? 1 : 2) === period;
  });
  const level = !pending(all) ? 'day' : !pending(group) ? 'period' : 'item';
  const message =
    level === 'day'
      ? 'Day cleared. Legend.'
      : level === 'period'
        ? ['Morning: done!', 'Afternoon: done!', 'Evening: done!'][period]
        : 'Nice!';
  return { id, personId: item.routine.person_id, itemId: item.id, message, level };
}
