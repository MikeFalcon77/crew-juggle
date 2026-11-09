/** The crew's calendar zone. Set once the crew is known; times are shown in it. */
let crewZone = 'UTC';
export function setCrewZone(zone: string) {
  crewZone = zone;
}
export function getCrewZone() {
  return crewZone;
}
export function browserZone() {
  return Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC';
}
export function zoneName(zone: string) {
  return zone.split('/').at(-1)!.replaceAll('_', ' ');
}
function clock(at: Date, zone: string) {
  return new Intl.DateTimeFormat('en-GB', {
    timeZone: zone,
    hour: '2-digit',
    minute: '2-digit',
    hourCycle: 'h23',
  }).format(at);
}
export function dateAt(at: Date, zone: string) {
  const parts = new Intl.DateTimeFormat('en-CA', {
    timeZone: zone,
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  }).formatToParts(at);
  const get = (type: string) => parts.find((p) => p.type === type)!.value;
  return `${get('year')}-${get('month')}-${get('day')}`;
}
function shortDate(date: string) {
  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
    timeZone: 'UTC',
  }).format(new Date(`${date}T12:00:00Z`));
}
/** "09:00 Lisbon", plus the local date when it is not the crew's date. */
export function localTime(at: string, zone: string, crewDate?: string) {
  const instant = new Date(at);
  const localDate = dateAt(instant, zone);
  const baseDate = crewDate || dateAt(instant, crewZone);
  const date = localDate !== baseDate ? ` (${shortDate(localDate)})` : '';
  return `${clock(instant, zone)} ${zoneName(zone)}${date}`;
}
/** A crew clock slot, plus the viewer's time when they are elsewhere. */
export function scheduledLabel(
  time: string,
  at: string | undefined,
  zone = crewZone,
  date?: string,
) {
  return zone === crewZone || !at ? time : `${time} · ${localTime(at, zone, date)}`;
}
export function completedLabel(at: string, zone = crewZone) {
  const crew = clock(new Date(at), crewZone);
  return zone === crewZone ? crew : `${crew} · ${localTime(at, zone)}`;
}
export function longDate(date: string) {
  return new Intl.DateTimeFormat('en-US', {
    weekday: 'long',
    month: 'long',
    day: 'numeric',
    timeZone: 'UTC',
  }).format(new Date(`${date}T12:00:00Z`));
}
export const WEEKDAYS = [
  { day: 1, short: 'Mo', long: 'Monday' },
  { day: 2, short: 'Tu', long: 'Tuesday' },
  { day: 3, short: 'We', long: 'Wednesday' },
  { day: 4, short: 'Th', long: 'Thursday' },
  { day: 5, short: 'Fr', long: 'Friday' },
  { day: 6, short: 'Sa', long: 'Saturday' },
  { day: 7, short: 'Su', long: 'Sunday' },
];
/** "Every day", "Weekdays", "Tue", "Mon, Wed, Fri". */
export function daysLabel(days: number[]) {
  const set = [...new Set(days)].sort();
  if (!set.length || set.length === 7) return 'Every day';
  if (set.join() === '1,2,3,4,5') return 'Weekdays';
  if (set.join() === '6,7') return 'Weekends';
  return set.map((d) => WEEKDAYS[d - 1].long.slice(0, 3)).join(', ');
}
