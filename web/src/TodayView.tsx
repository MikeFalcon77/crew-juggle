import { useState } from 'react';
import { ArrowRight, Check } from 'lucide-react';
import { ShoppingBag } from 'lucide-react';
import PersonAvatar from './PersonAvatar';
import { tracks, type CardGroup } from './today';
import { useTodayTracks } from './useTodayTracks';
import type { Day, Occurrence } from './types';
import type { AppRoute } from './routes';
import { scheduledLabel, getCrewZone, longDate, zoneName } from './time';
import './today.css';
import type { Celebration } from './celebration';
export default function TodayView({
  day,
  crewName,
  celebration,
  routinesLoading,
  routinesError,
  online,
  saving,
  onMark,
  onNavigate,
  zone = getCrewZone(),
  person = '',
  onPerson,
}: {
  day?: Day;
  crewName: string;
  celebration?: Celebration | null;
  routinesLoading: boolean;
  routinesError: boolean;
  online: boolean;
  saving: string | null;
  onMark: (item: Occurrence) => void;
  onNavigate: (target: AppRoute) => void;
  zone?: string;
  person?: string;
  onPerson: (person: string) => void;
}) {
  const [expanded, setExpanded] = useState<Partial<Record<CardGroup, boolean>>>({});
  const { today, people, avatars, cards, messages, incomplete } = useTodayTracks({
    day,
    routinesLoading,
    routinesError,
    online,
  });
  const data = cards.filter((card) => !person || card.personId === person || !card.personId);
  const groups: [CardGroup, string, string][] = [
    ['attention', 'Needs attention', 'Overdue or waiting for a look'],
    ['good', 'Wins', 'Small victories of the crew'],
    ['next', 'Up next', 'What is coming and what to buy'],
  ];
  return (
    <>
      <div className="section-heading">
        <div>
          <span className="eyebrow">{crewName.toUpperCase()} · TODAY</span>
          <h1>Today</h1>
          <p>
            {today ? longDate(today) : ''} · {zoneName(getCrewZone())} time
          </p>
        </div>
        <div className="today-track-links">
          {tracks.map((track) => (
            <button
              className="secondary"
              key={track.id}
              onClick={() => onNavigate({ ...track.target, person: person || undefined })}
            >
              <track.Icon size={16} />
              {track.label}
            </button>
          ))}
        </div>
      </div>
      <div className="today-people" role="group" aria-label="Whose tasks">
        <button className={!person ? 'active' : ''} onClick={() => onPerson('')}>
          Whole crew
        </button>
        {people.map((p) => (
          <button
            key={p.id}
            className={person === p.id ? 'active' : ''}
            onClick={() => onPerson(p.id)}
          >
            <PersonAvatar person={p} meta={avatars?.[p.id]} />
            <span>{p.name}</span>
          </button>
        ))}
      </div>
      {celebration && (
        <div
          className={`celebration celebration-${celebration.level}`}
          key={celebration.id}
          role="status"
        >
          ✓ {celebration.message}
        </div>
      )}
      <div className="today-source-status" aria-label="Status">
        {messages.map((message) => (
          <p role="status" key={message}>
            {message}
          </p>
        ))}
      </div>
      <div className="today-grid">
        {groups.map(([id, title, subtitle]) => {
          const cards = data
            .filter((c) => c.group === id)
            .sort((a, b) =>
              id === 'next'
                ? a.sort.localeCompare(b.sort) || a.id.localeCompare(b.id)
                : a.track !== b.track
                  ? a.track === 'routines'
                    ? -1
                    : 1
                  : b.sort.localeCompare(a.sort) || a.id.localeCompare(b.id),
            );
          const shown = expanded[id] ? cards : cards.slice(0, 3);
          return (
            <section className={`today-block today-${id}`} key={id} aria-label={title}>
              <header>
                <div>
                  <h2>
                    {title}
                    <span>{cards.length}</span>
                  </h2>
                  <p>{subtitle}</p>
                </div>
              </header>
              {shown.map((card) => {
                const p = people.find((p) => p.id === card.personId);
                const actionable = Boolean(card.action) && !routinesError && online && !saving;
                return (
                  <article className="today-card" key={card.id}>
                    <div className="today-card-person">
                      {p ? (
                        <PersonAvatar person={p} meta={avatars?.[p.id]} />
                      ) : (
                        <span className="track-initial">
                          <ShoppingBag size={18} />
                        </span>
                      )}
                      <div>
                        <strong>{p?.name || card.personName || 'Crew'}</strong>
                        <small>{tracks.find((track) => track.id === card.track)?.label}</small>
                      </div>
                    </div>
                    <h3>{card.title}</h3>
                    <p>{card.detail}</p>
                    {card.action && (
                      <div className="today-quick-action">
                        <span>
                          {scheduledLabel(
                            card.action.routine.time,
                            day?.scheduled_times[card.action.routine.time],
                            zone,
                            day?.date,
                          )}
                        </span>
                        <button
                          className="primary"
                          disabled={!actionable}
                          aria-label={`Check off ${card.action.routine.name} for ${p?.name || card.personId}`}
                          onClick={() => onMark(card.action!)}
                        >
                          <Check size={15} />
                          {saving === card.action.id ? 'Saving…' : 'Check off'}
                        </button>
                      </div>
                    )}
                    <button
                      className="text-button today-detail-link"
                      onClick={() => onNavigate(card.target)}
                    >
                      Open {card.track === 'routines' ? 'routines' : 'shopping'}
                      <ArrowRight size={15} />
                    </button>
                  </article>
                );
              })}
              {!cards.length && (
                <p className="today-empty">
                  {id === 'attention'
                    ? incomplete
                      ? 'Still loading part of the picture.'
                      : 'Nothing on fire. Nice.'
                    : id === 'good'
                      ? 'Wins show up here.'
                      : 'Nothing coming up.'}
                </p>
              )}
              {cards.length > 3 && (
                <button
                  className="text-button"
                  onClick={() => setExpanded((e) => ({ ...e, [id]: !e[id] }))}
                >
                  {expanded[id] ? 'Show less' : `${cards.length - 3} more`}
                </button>
              )}
            </section>
          );
        })}
      </div>
    </>
  );
}
