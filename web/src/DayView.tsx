import {
  Check,
  Circle,
  Clock3,
  Link2,
  LockKeyhole,
  Pencil,
  Sun,
  Sunrise,
  Sunset,
} from 'lucide-react';
import { completedLabel, scheduledLabel, getCrewZone } from './time';
import RoutineFace from './RoutineFace';
import PersonAvatar from './PersonAvatar';
import type { Celebration } from './celebration';
import type { Day, Occurrence } from './types';

export function formatTime(value: string) {
  return new Intl.DateTimeFormat('en-GB', {
    timeZone: getCrewZone(),
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(value));
}
export function canMark(item: Occurrence, items: Occurrence[]) {
  return (
    !item.routine.prerequisite ||
    items.some((i) => i.routine.id === item.routine.prerequisite && i.status === 'done')
  );
}
export const periods = [
  {
    id: 'morning',
    label: 'Morning',
    range: '00:00–11:59',
    min: '00:00',
    max: '12:00',
    Icon: Sunrise,
  },
  { id: 'day', label: 'Afternoon', range: '12:00–17:59', min: '12:00', max: '18:00', Icon: Sun },
  {
    id: 'evening',
    label: 'Evening',
    range: '18:00–23:59',
    min: '18:00',
    max: '24:00',
    Icon: Sunset,
  },
];
export function DayView({
  day,
  myUid,
  filter,
  readonly,
  saving,
  disabled,
  onMark,
  zone = getCrewZone(),
  celebration,
  personId,
  highlightedRoutineId,
  onEdit,
}: {
  day: Day;
  myUid: string;
  filter: string;
  readonly: boolean;
  saving: string | null;
  disabled: boolean;
  onMark: (item: Occurrence) => void;
  zone?: string;
  celebration?: Celebration | null;
  personId?: string;
  highlightedRoutineId?: string;
  onEdit?: (item: Occurrence) => void;
}) {
  return (
    <div className="tracks-grid">
      {day.people
        .filter((person) => !personId || person.id === personId)
        .map((person) => {
          const all = day.items.filter(
            (i) => i.routine.person_id === person.id && i.status !== 'cancelled',
          );
          const completed = all.filter((i) => i.status === 'done').length;
          const visible = all.filter(
            (i) =>
              filter === 'all' ||
              (filter === 'left' ? i.status === 'pending' : i.status === 'done'),
          );
          return (
            <section className={`track-card tone-${person.tone}`} key={person.id}>
              <div className="track-head">
                <PersonAvatar person={person} meta={day.avatars?.[person.id]} />
                <div>
                  <h2>{person.name}</h2>
                  <p>
                    {all.length
                      ? `${all.length - completed} to go · ${completed} done`
                      : 'Nothing today'}
                  </p>
                </div>
                <span className="track-counter">
                  {completed}
                  <span>/{all.length}</span>
                </span>
              </div>
              {celebration?.personId === person.id && (
                <div
                  className={`celebration celebration-${celebration.level}`}
                  key={celebration.id}
                  role="status"
                >
                  <span aria-hidden="true">✦</span>
                  {celebration.message}
                  <span aria-hidden="true">✧</span>
                </div>
              )}
              <div className="track-progress">
                <span style={{ width: `${all.length ? (completed / all.length) * 100 : 0}%` }} />
              </div>
              <div className="track-body">
                {periods.map((period) => {
                  const items = visible
                    .filter((i) => i.routine.time >= period.min && i.routine.time < period.max)
                    .sort(
                      (a, b) =>
                        a.routine.time.localeCompare(b.routine.time) ||
                        a.routine.group.localeCompare(b.routine.group) ||
                        Number(Boolean(a.routine.prerequisite)) -
                          Number(Boolean(b.routine.prerequisite)),
                    );
                  if (!items.length) return null;
                  const slots = new Map<string, Occurrence[]>();
                  items.forEach((item) => {
                    const key = `${item.routine.time}|${item.routine.group}`;
                    slots.set(key, [...(slots.get(key) || []), item]);
                  });
                  return (
                    <section
                      className={`time-period ${period.id}`}
                      key={period.id}
                      aria-label={`${period.label}, ${person.name}`}
                    >
                      <div className="period-head">
                        <span className="period-icon">
                          <period.Icon size={19} />
                        </span>
                        <h3>{period.label}</h3>
                        <span className="period-range">{period.range}</span>
                      </div>
                      {[...slots.entries()].map(([key, slotItems]) => {
                        const p = slotItems[0].routine;
                        return (
                          <div className="chore-group" key={key}>
                            <div className="group-head">
                              {slotItems.some((i) => i.routine.prerequisite) && <Link2 size={13} />}
                              <h4>{p.group || period.label}</h4>
                              <span>
                                <Clock3 size={12} />
                                {scheduledLabel(
                                  p.time,
                                  day.scheduled_times[p.time],
                                  zone,
                                  day.date,
                                )}
                              </span>
                            </div>
                            {slotItems.map((item) => {
                              const done = item.status === 'done';
                              const locked = !done && !canMark(item, day.items);
                              const undoLocked =
                                done &&
                                day.items.some(
                                  (i) =>
                                    i.routine.prerequisite === item.routine.id &&
                                    i.status === 'done',
                                );
                              const actor =
                                item.completed_by === myUid
                                  ? 'You'
                                  : day.people.find((p) => p.account === item.completed_by)?.name ||
                                    'Someone';
                              const content = (
                                <>
                                  <RoutineFace
                                    mood={day.appearances?.[item.routine.id]?.mood}
                                    celebrating={celebration?.itemId === item.id}
                                  />
                                  <span
                                    className={`check-box ${saving === item.id ? 'waiting' : ''}`}
                                  >
                                    {saving === item.id ? (
                                      <span className="spinner small" />
                                    ) : done ? (
                                      <Check size={16} strokeWidth={3} />
                                    ) : locked ? (
                                      <LockKeyhole size={13} />
                                    ) : (
                                      <Circle size={5} fill="currentColor" />
                                    )}
                                  </span>
                                  <span className="chore-copy">
                                    <span className="chore-name">{item.routine.name}</span>
                                    {item.routine.note && (
                                      <span className="chore-note">{item.routine.note}</span>
                                    )}
                                    {locked && (
                                      <span className="dependency">
                                        After{' '}
                                        {day.items.find(
                                          (i) => i.routine.id === item.routine.prerequisite,
                                        )?.routine.name || 'the first step'}
                                      </span>
                                    )}
                                    {done && (
                                      <span className="done-by">
                                        <Check size={11} />
                                        {actor} ·{' '}
                                        {item.completed_at &&
                                          completedLabel(item.completed_at, zone)}
                                      </span>
                                    )}
                                  </span>
                                </>
                              );
                              const unavailable =
                                disabled || readonly || Boolean(saving) || locked || undoLocked;
                              return (
                                <div
                                  className={`chore-entry ${!readonly && onEdit ? 'editable' : ''}`}
                                  key={item.id}
                                >
                                  {done ? (
                                    <div
                                      className={`chore-row done ${item.routine.id === highlightedRoutineId ? 'record-highlight' : ''}`}
                                      role="group"
                                      aria-label={`Done: ${item.routine.name}, ${person.name}, ${p.time}`}
                                      key={item.id}
                                    >
                                      {content}
                                      {!readonly && (
                                        <button
                                          type="button"
                                          className="undo-button"
                                          disabled={unavailable}
                                          onClick={() => onMark(item)}
                                          aria-label={`Undo: ${item.routine.name}, ${person.name}, ${p.time}`}
                                          title={
                                            undoLocked ? 'Undo the second step first' : undefined
                                          }
                                        >
                                          Undo
                                        </button>
                                      )}
                                    </div>
                                  ) : (
                                    <button
                                      type="button"
                                      className={`chore-row ${locked ? 'locked' : ''} ${item.routine.id === highlightedRoutineId ? 'record-highlight' : ''}`}
                                      key={item.id}
                                      disabled={unavailable}
                                      onClick={() => onMark(item)}
                                      aria-label={`Check off: ${item.routine.name}, ${person.name}, ${p.time}`}
                                      aria-pressed={false}
                                      title={locked ? 'Check off the first step first' : undefined}
                                    >
                                      {content}
                                    </button>
                                  )}
                                  {!readonly && onEdit && (
                                    <button
                                      type="button"
                                      className="icon-button chore-edit"
                                      aria-label={`Edit: ${item.routine.name}, ${person.name}, ${p.time}`}
                                      title="Edit routine"
                                      disabled={disabled || Boolean(saving)}
                                      onClick={() => onEdit(item)}
                                    >
                                      <Pencil size={16} />
                                    </button>
                                  )}
                                </div>
                              );
                            })}
                          </div>
                        );
                      })}
                    </section>
                  );
                })}
                {visible.length === 0 && (
                  <div className="empty-track">
                    <Check size={24} />
                    <p>
                      {all.length && filter === 'left'
                        ? 'All done. Go touch grass.'
                        : 'Nothing here yet'}
                    </p>
                  </div>
                )}
              </div>
            </section>
          );
        })}
    </div>
  );
}
