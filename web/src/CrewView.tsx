import { useState } from 'react';
import {
  ArrowRight,
  Camera,
  Copy,
  KeyRound,
  Pencil,
  RefreshCw,
  UserPlus,
  X,
  Check,
} from 'lucide-react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { api, apiAt } from './api';
import AvatarEditor from './AvatarEditor';
import PersonAvatar from './PersonAvatar';
import { zoneName } from './time';
import type { AvatarMeta, Me, Person } from './types';

const tones = [
  { id: 'blue', name: 'Blue' },
  { id: 'lilac', name: 'Lilac' },
  { id: 'green', name: 'Green' },
  { id: 'peach', name: 'Peach' },
];

export default function CrewView({
  me,
  disabled,
  onError,
}: {
  me: Me;
  disabled: boolean;
  onError: (message: string) => void;
}) {
  const client = useQueryClient();
  const crew = me.crew!;
  const owner = crew.role === 'owner';
  const query = useQuery({
    queryKey: ['people'],
    queryFn: () =>
      api<{ people: Person[]; today: string; avatars?: Record<string, AvatarMeta> }>('/people'),
    refetchInterval: 5000,
  });
  const [photo, setPhoto] = useState<Person | null>(null);
  const [editing, setEditing] = useState<Person | 'new' | null>(null);
  const [newId, setNewId] = useState('');
  const [operationId, setOperationId] = useState('');
  const [name, setName] = useState('');
  const [tone, setTone] = useState('green');
  const [message, setMessage] = useState('');
  const [copied, setCopied] = useState(false);
  const [crewName, setCrewName] = useState(crew.name);
  const [crewZone, setCrewZone] = useState(crew.timezone);
  const inviteLink = `${location.origin}/join?code=${encodeURIComponent(crew.invite_code)}`;

  const save = useMutation({
    mutationFn: () =>
      api('/people', 'PUT', {
        operation_id: operationId,
        expected_revision: editing && editing !== 'new' ? editing.revision : 0,
        person: {
          id: editing && editing !== 'new' ? editing.id : newId,
          name,
          tone,
          revision: 0,
          created_date: query.data!.today,
        },
      }),
    onSuccess: async () => {
      await Promise.all([
        client.invalidateQueries({ queryKey: ['people'] }),
        client.invalidateQueries({ queryKey: ['day'] }),
        client.invalidateQueries({ queryKey: ['schedule'] }),
      ]);
      setEditing(null);
    },
    onError: (error) => {
      setMessage(error.message);
      onError(error.message);
      void client.invalidateQueries({ queryKey: ['people'] });
    },
  });
  const rotate = useMutation({
    mutationFn: () => apiAt<Me>('/juggle-crews/v1/crew/invite-code', 'POST'),
    onSuccess: (fresh) => client.setQueryData(['me'], fresh),
    onError: (error) => onError(error.message),
  });
  const update = useMutation({
    mutationFn: () =>
      apiAt<Me>('/juggle-crews/v1/crew', 'PUT', { name: crewName, timezone: crewZone }),
    onSuccess: async (fresh) => {
      client.setQueryData(['me'], fresh);
      await client.invalidateQueries();
    },
    onError: (error) => onError(error.message),
  });
  function open(person: Person | 'new') {
    setEditing(person);
    setNewId(`p-${crypto.randomUUID().slice(0, 8)}`);
    setOperationId(crypto.randomUUID());
    setName(person === 'new' ? '' : person.name);
    setTone(person === 'new' ? 'green' : person.tone);
    setMessage('');
  }
  async function copy() {
    try {
      await navigator.clipboard.writeText(inviteLink);
      setCopied(true);
      setTimeout(() => setCopied(false), 1800);
    } catch {
      onError("Couldn't copy. Select the code and copy it by hand.");
    }
  }
  return (
    <>
      <div className="section-heading">
        <div>
          <span className="eyebrow">YOUR CREW</span>
          <h1>{crew.name}</h1>
          <p>
            Everyone here shares routines and the shopping list. Nobody outside the crew sees a
            thing.
          </p>
        </div>
        <button className="primary" onClick={() => open('new')} disabled={disabled || !query.data}>
          <UserPlus size={18} />
          Add someone without an account
        </button>
      </div>
      <div className="settings-grid crew-cards">
        <section className="settings-card invite-card">
          <h2>
            <KeyRound size={18} /> Invite your crew
          </h2>
          <p>Send the link or the code. Whoever opens it signs in and joins {crew.name}.</p>
          <div className="invite-code" aria-label="Invite code">
            {crew.invite_code}
          </div>
          <div className="invite-actions">
            <button className="secondary" onClick={() => void copy()}>
              {copied ? <Check size={16} /> : <Copy size={16} />}
              {copied ? 'Copied' : 'Copy invite link'}
            </button>
            {owner && (
              <button
                className="text-button"
                disabled={disabled || rotate.isPending}
                onClick={() => rotate.mutate()}
              >
                <RefreshCw size={14} /> New code
              </button>
            )}
          </div>
          {owner && <small>A new code makes the old one stop working.</small>}
        </section>
        {owner && (
          <section className="settings-card">
            <h2>Crew settings</h2>
            <form
              onSubmit={(event) => {
                event.preventDefault();
                update.mutate();
              }}
            >
              <label>
                Crew name
                <input
                  value={crewName}
                  onChange={(event) => setCrewName(event.target.value)}
                  maxLength={80}
                  required
                />
              </label>
              <label>
                Crew time zone
                <select value={crewZone} onChange={(event) => setCrewZone(event.target.value)}>
                  {[...new Set([crew.timezone, ...Intl.supportedValuesOf('timeZone')])].map(
                    (zone) => (
                      <option value={zone} key={zone}>
                        {zoneName(zone)} · {zone}
                      </option>
                    ),
                  )}
                </select>
              </label>
              <button
                className="primary"
                disabled={
                  disabled ||
                  update.isPending ||
                  (crewName === crew.name && crewZone === crew.timezone)
                }
              >
                {update.isPending ? 'Saving…' : 'Save crew'}
              </button>
            </form>
          </section>
        )}
      </div>
      {query.isPending && (
        <div className="loading">
          <span className="spinner" />
          Loading…
        </div>
      )}
      {query.isError && <div className="notice error">{query.error.message}</div>}
      <div className="people-grid">
        {query.data?.people.map((person) => (
          <section className={`person-card tone-${person.tone}`} key={person.id}>
            <button
              className="avatar-button"
              aria-label={`Photo: ${person.name}`}
              onClick={() => setPhoto(person)}
              disabled={disabled || query.isError}
            >
              <PersonAvatar person={person} meta={query.data?.avatars?.[person.id]} />
              <span className="avatar-camera">
                <Camera size={13} />
              </span>
            </button>
            <div>
              <h2>
                {person.name}
                {person.account === me.uid && <span className="you-badge">you</span>}
              </h2>
              <p>
                {person.account
                  ? crew.members.find((m) => m.uid === person.account)?.role === 'owner'
                    ? 'Owner · signs in'
                    : 'Signs in'
                  : 'No account · has chores anyway'}
              </p>
            </div>
            <button
              className="icon-button"
              aria-label={`Edit: ${person.name}`}
              onClick={() => open(person)}
              disabled={disabled}
            >
              <Pencil size={17} />
            </button>
          </section>
        ))}
      </div>
      {photo && (
        <AvatarEditor
          person={photo}
          meta={query.data?.avatars?.[photo.id]}
          disabled={disabled}
          onClose={() => setPhoto(null)}
          onError={onError}
        />
      )}
      {editing && (
        <div
          className="modal-backdrop"
          onClick={(event) => {
            if (event.target === event.currentTarget && !save.isPending) setEditing(null);
          }}
        >
          <section
            className="editor person-editor"
            role="dialog"
            aria-modal="true"
            aria-labelledby="person-title"
          >
            <div className="editor-head">
              <div>
                <span className="eyebrow">CREW</span>
                <h2 id="person-title">
                  {editing === 'new' ? 'Add someone' : `Edit ${editing.name}`}
                </h2>
              </div>
              <button
                className="icon-button"
                aria-label="Close"
                onClick={() => setEditing(null)}
                disabled={save.isPending}
              >
                <X size={20} />
              </button>
            </div>
            {editing === 'new' && (
              <p className="editor-intro">
                For crewmates who never sign in: a cat, a little brother, the office plant. People
                with an account appear here on their own when they join.
              </p>
            )}
            <form
              onSubmit={(event) => {
                event.preventDefault();
                save.mutate();
              }}
            >
              <label>
                Name
                <input
                  value={name}
                  onChange={(event) => setName(event.target.value)}
                  maxLength={80}
                  placeholder="Mochi"
                  required
                  autoFocus
                  disabled={save.isPending}
                />
              </label>
              <span className="field-label">Card color</span>
              <div className="tone-picker">
                {tones.map((item) => (
                  <button
                    type="button"
                    key={item.id}
                    className={`tone-choice tone-${item.id} ${tone === item.id ? 'selected' : ''}`}
                    onClick={() => setTone(item.id)}
                    aria-pressed={tone === item.id}
                    disabled={save.isPending}
                  >
                    <i />
                    {item.name}
                  </button>
                ))}
              </div>
              {message && (
                <div className="notice error" role="alert">
                  {message}
                </div>
              )}
              <div className="editor-actions">
                <button
                  type="button"
                  className="secondary"
                  onClick={() => setEditing(null)}
                  disabled={save.isPending}
                >
                  Cancel
                </button>
                <button
                  className="primary"
                  type="submit"
                  disabled={save.isPending || disabled || !name.trim()}
                >
                  {save.isPending ? 'Saving…' : 'Save'}
                  <ArrowRight size={16} />
                </button>
              </div>
            </form>
          </section>
        </div>
      )}
    </>
  );
}
