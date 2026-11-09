import { CalendarDays, ChevronLeft, ChevronRight, Plus } from 'lucide-react';
import { useState } from 'react';
import { DayView } from './DayView';
import DailySummary from './DailySummary';
import PersonAvatar from './PersonAvatar';
import ScheduleView from './ScheduleView';
import RoutineEditorLoader, { type RoutineEditRequest } from './RoutineEditorLoader';
import { longDate, zoneName, getCrewZone } from './time';
import type { AppRoute } from './routes';
import type { Celebration } from './celebration';
import type { Day, Occurrence } from './types';

export default function RoutinesPage({
  route,
  day,
  dayPending,
  myUid,
  zone,
  disabled,
  saving,
  celebration,
  onMark,
  onNavigate,
  onError,
}: {
  route: AppRoute;
  day?: Day;
  dayPending: boolean;
  myUid: string;
  zone: string;
  disabled: boolean;
  saving: string | null;
  celebration: Celebration | null;
  onMark: (item: Occurrence) => void;
  onNavigate: (route: AppRoute) => void;
  onError: (message: string) => void;
}) {
  const tab = route.tab || 'today';
  const isHistory = tab === 'history';
  const [filter, setFilter] = useState('all');
  const [editing, setEditing] = useState<RoutineEditRequest | null>(null);
  const [notice, setNotice] = useState('');
  const shown =
    day && route.person
      ? { ...day, items: day.items.filter((i) => i.routine.person_id === route.person) }
      : day;
  const items = shown?.items.filter((i) => i.status !== 'cancelled') || [];
  const remaining = items.filter((i) => i.status !== 'done').length;
  const historyDate = route.date || day?.today || '';
  const shift = (days: number) => {
    const next = new Date(`${historyDate}T12:00:00Z`);
    next.setUTCDate(next.getUTCDate() + days);
    onNavigate({ ...route, tab: 'history', date: next.toISOString().slice(0, 10) });
  };
  return (
    <>
      <div className="section-heading">
        <div>
          <span className="eyebrow">{isHistory ? 'EVERY DAY IS KEPT' : 'ROUTINES'}</span>
          <h1>{isHistory ? 'History' : tab === 'schedule' ? 'Schedule' : 'Routines'}</h1>
          <p>
            {isHistory
              ? 'Who checked off what, and when. Past days never change.'
              : tab === 'schedule'
                ? 'Who does what, when and on which days.'
                : "Today's tasks for the whole crew."}
          </p>
        </div>
        <div className="health-heading-actions">
          {tab === 'today' && (
            <button
              className="primary"
              disabled={disabled || Boolean(saving) || !day?.people.length}
              onClick={() => {
                setNotice('');
                setEditing({ kind: 'new', personId: route.person });
              }}
            >
              <Plus size={18} />
              New routine
            </button>
          )}
          {day && tab !== 'schedule' && (
            <div className="date-card">
              <CalendarDays size={19} />
              <div>
                <strong>{longDate(isHistory ? historyDate : day.date)}</strong>
                <span>{zoneName(getCrewZone())} time</span>
              </div>
            </div>
          )}
        </div>
      </div>
      <nav className="track-tabs" aria-label="Routines sections">
        {[
          ['today', 'Today'],
          ['schedule', 'Schedule'],
          ['history', 'History'],
        ].map(([id, label]) => (
          <button
            key={id}
            className={tab === id ? 'active' : ''}
            aria-current={tab === id ? 'page' : undefined}
            onClick={() =>
              onNavigate({
                page: 'routines',
                tab: id,
                person: route.person,
                date: id === 'history' ? day?.today : undefined,
              })
            }
          >
            {label}
          </button>
        ))}
      </nav>
      {tab === 'schedule' ? (
        <ScheduleView zone={zone} disabled={disabled} onError={onError} />
      ) : (
        <>
          {notice && (
            <p className="notice" role="status">
              {notice}
            </p>
          )}
          {editing && (
            <RoutineEditorLoader
              request={editing}
              disabled={disabled || Boolean(saving)}
              onClose={() => setEditing(null)}
              onError={onError}
              onSaved={(effective) =>
                setNotice(
                  `Saved. It applies from ${longDate(effective)}; done tasks stay as they were.`,
                )
              }
            />
          )}
          {day && (
            <div className="today-people" role="group" aria-label="Whose routines">
              <button
                className={!route.person ? 'active' : ''}
                onClick={() => onNavigate({ ...route, person: undefined, record: undefined })}
              >
                Whole crew
              </button>
              {day.people.map((person) => (
                <button
                  key={person.id}
                  className={route.person === person.id ? 'active' : ''}
                  onClick={() => onNavigate({ ...route, person: person.id, record: undefined })}
                >
                  <PersonAvatar person={person} meta={day.avatars?.[person.id]} />
                  {person.name}
                </button>
              ))}
            </div>
          )}
          {isHistory && day && (
            <div className="history-picker">
              <button
                className="icon-button"
                aria-label="Previous day"
                disabled={historyDate <= day.first_date}
                onClick={() => shift(-1)}
              >
                <ChevronLeft size={18} />
              </button>
              <input
                aria-label="History date"
                type="date"
                value={historyDate}
                max={day.today}
                min={day.first_date}
                onChange={(e) => onNavigate({ ...route, tab: 'history', date: e.target.value })}
              />
              <button
                className="icon-button"
                aria-label="Next day"
                disabled={historyDate >= day.today}
                onClick={() => shift(1)}
              >
                <ChevronRight size={18} />
              </button>
            </div>
          )}
          {shown && (
            <>
              <DailySummary day={shown} zone={zone} />
              <div className="list-toolbar">
                <h2>
                  {isHistory ? 'That day' : "Today's plan"}
                  <span>{items.length}</span>
                </h2>
                <div className="filter-tabs" aria-label="Filter">
                  {[
                    ['all', 'All'],
                    ['left', 'To do'],
                    ['done', 'Done'],
                  ].map(([id, label]) => (
                    <button
                      key={id}
                      className={filter === id ? 'active' : ''}
                      onClick={() => setFilter(id)}
                    >
                      {label}
                      {id === 'left' && <span>{remaining}</span>}
                    </button>
                  ))}
                </div>
              </div>
              <DayView
                day={day!}
                personId={route.person}
                highlightedRoutineId={
                  route.record?.startsWith('occurrence:') ? route.record.slice(11) : undefined
                }
                myUid={myUid}
                filter={filter}
                readonly={isHistory}
                saving={saving}
                disabled={disabled}
                zone={zone}
                celebration={celebration}
                onMark={onMark}
                onEdit={
                  isHistory
                    ? undefined
                    : (item) => {
                        setNotice('');
                        setEditing({ kind: 'edit', routineId: item.routine.id });
                      }
                }
              />
            </>
          )}
          {dayPending && (
            <div className="loading">
              <span className="spinner" />
              Getting today ready…
            </div>
          )}
        </>
      )}
    </>
  );
}
