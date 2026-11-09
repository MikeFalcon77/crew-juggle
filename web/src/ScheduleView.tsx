import { useState } from 'react';
import { Link2, Pencil, Plus } from 'lucide-react';
import { useQuery } from '@tanstack/react-query';
import { api } from './api';
import RoutineFace from './RoutineFace';
import PersonAvatar from './PersonAvatar';
import AppearanceEditor from './AppearanceEditor';
import { periods } from './DayView';
import { daysLabel, getCrewZone, localTime } from './time';
import type { Routine, Schedule, Version } from './types';
import RoutineEditor from './RoutineEditor';
export { nextDate } from './RoutineEditor';

export default function ScheduleView({
  disabled,
  onError,
  zone = getCrewZone(),
}: {
  disabled: boolean;
  onError: (message: string) => void;
  zone?: string;
}) {
  const query = useQuery({
    queryKey: ['schedule'],
    queryFn: () => api<Schedule>('/schedule'),
    refetchInterval: 5000,
  });
  const [editing, setEditing] = useState<Version | 'new' | null>(null);
  const [appearance, setAppearance] = useState<Routine | null>(null);
  const [personId, setPersonId] = useState('');
  if (query.isPending)
    return (
      <div className="loading">
        <span className="spinner" />
        Loading routines…
      </div>
    );
  if (query.isError) return <div className="notice error">{query.error.message}</div>;
  const data = query.data;
  const nameOf = (id: string | null) =>
    data.versions.find((v) => v.routine.id === id)?.routine.name;
  function add(id?: string) {
    setPersonId(id || data.people[0]?.id || '');
    setEditing('new');
  }
  return (
    <>
      <div className="list-toolbar">
        <h2>
          Every routine <span>{data.versions.length}</span>
        </h2>
        <button
          className="primary"
          onClick={() => add()}
          disabled={disabled || !data.people.length}
        >
          <Plus size={18} />
          New routine
        </button>
      </div>
      <div className="schedule-grid">
        {data.people.map((person) => {
          const versions = data.versions.filter((v) => v.routine.person_id === person.id);
          return (
            <section className={`schedule-card tone-${person.tone}`} key={person.id}>
              <div className="schedule-title">
                <PersonAvatar person={person} meta={data.avatars?.[person.id]} small />
                <h2>{person.name}</h2>
              </div>
              {periods.map((period) => {
                const entries = versions
                  .filter((v) => v.routine.time >= period.min && v.routine.time < period.max)
                  .sort(
                    (a, b) =>
                      a.routine.time.localeCompare(b.routine.time) ||
                      Number(Boolean(a.routine.prerequisite)) -
                        Number(Boolean(b.routine.prerequisite)),
                  );
                if (!entries.length) return null;
                return (
                  <section
                    className="schedule-period"
                    key={period.id}
                    aria-label={`${period.label}, ${person.name}`}
                  >
                    <div className="period-head">
                      <span className="period-icon">
                        <period.Icon size={18} />
                      </span>
                      <h3>{period.label}</h3>
                      <span className="period-range">{period.range}</span>
                    </div>
                    {entries.map((version) => {
                      const r = version.routine;
                      const displayDate = [data.today, version.effective_from, r.start_date]
                        .sort()
                        .at(-1)!;
                      const at = data.scheduled_times[`${displayDate}/${r.time}`];
                      return (
                        <div className={`schedule-row ${!r.enabled ? 'inactive' : ''}`} key={r.id}>
                          <button
                            className="face-button"
                            aria-label={`Mood: ${r.name}`}
                            disabled={disabled}
                            onClick={() => setAppearance(r)}
                          >
                            <RoutineFace mood={data.appearances?.[r.id]?.mood} />
                          </button>
                          <div className="schedule-clock">
                            {r.time}
                            <small>{daysLabel(r.days)}</small>
                            {r.group && <small>{r.group}</small>}
                          </div>
                          <div className="schedule-copy">
                            <strong>{r.name}</strong>
                            {r.prerequisite && (
                              <span className="schedule-step">
                                <Link2 size={12} /> after {nameOf(r.prerequisite)}
                              </span>
                            )}
                            {zone !== getCrewZone() && at && (
                              <span className="schedule-local-time">
                                {localTime(at, zone, displayDate)}
                              </span>
                            )}
                            {r.note && <p>{r.note}</p>}
                            {r.end_date && <small>Until {r.end_date}</small>}
                            {!r.enabled && <small>Off</small>}
                            {version.effective_from > data.today && (
                              <small>Changes on {version.effective_from}</small>
                            )}
                          </div>
                          <button
                            className="icon-button"
                            aria-label={`Edit: ${r.name}`}
                            onClick={() => setEditing(version)}
                            disabled={disabled}
                          >
                            <Pencil size={16} />
                          </button>
                        </div>
                      );
                    })}
                  </section>
                );
              })}
              {!versions.length && (
                <div className="empty-track">
                  <p>Nothing on {person.name}'s plate yet</p>
                  <button
                    className="text-button"
                    onClick={() => add(person.id)}
                    disabled={disabled}
                  >
                    <Plus size={14} />
                    Add a routine
                  </button>
                </div>
              )}
            </section>
          );
        })}
      </div>
      {appearance && (
        <AppearanceEditor
          routine={appearance}
          appearance={data.appearances?.[appearance.id]}
          disabled={disabled}
          onClose={() => setAppearance(null)}
          onError={onError}
        />
      )}
      {editing && (
        <RoutineEditor
          key={editing === 'new' ? 'new' : editing.routine.id}
          version={editing === 'new' ? null : editing}
          versions={data.versions}
          people={data.people}
          avatars={data.avatars}
          initialPersonId={personId}
          today={data.today}
          disabled={disabled}
          onClose={() => setEditing(null)}
          onError={onError}
        />
      )}
    </>
  );
}
