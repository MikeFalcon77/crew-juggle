import { useState } from 'react';
import { ArrowRight, LogOut, Plus, Ticket } from 'lucide-react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { apiAt } from './api';
import { browserZone, zoneName } from './time';
import type { Me } from './types';

/** A signed-in account without a crew: create one, or join with a code. */
export default function Onboarding({
  me,
  initialCode,
  onSignOut,
}: {
  me: Me;
  initialCode?: string;
  onSignOut: () => void;
}) {
  const client = useQueryClient();
  const [mode, setMode] = useState<'create' | 'join'>(initialCode ? 'join' : 'create');
  const [displayName, setDisplayName] = useState(me.name);
  const [crewName, setCrewName] = useState('');
  const [timezone, setTimezone] = useState(browserZone());
  const [code, setCode] = useState(initialCode || '');
  const [error, setError] = useState('');
  const submit = useMutation({
    mutationFn: () =>
      mode === 'create'
        ? apiAt<Me>('/juggle-crews/v1/crews', 'POST', {
            name: crewName,
            display_name: displayName,
            timezone,
          })
        : apiAt<Me>('/juggle-crews/v1/memberships', 'POST', {
            invite_code: code,
            display_name: displayName,
          }),
    onSuccess: async (fresh) => {
      client.setQueryData(['me'], fresh);
      history.replaceState({}, '', '/today');
      await client.invalidateQueries();
    },
    onError: (e) => setError(e.message),
  });
  return (
    <div className="login-page">
      <div className="login-card onboarding">
        <span className="eyebrow">WELCOME, {me.name.toUpperCase()}</span>
        <h1>{mode === 'create' ? 'Start a crew' : 'Join your crew'}</h1>
        <p>
          A crew is your group: a flat, a team, a club. Its routines and shopping list are only
          visible to its members.
        </p>
        <div className="track-tabs" role="tablist">
          <button
            role="tab"
            aria-selected={mode === 'create'}
            className={mode === 'create' ? 'active' : ''}
            onClick={() => setMode('create')}
          >
            <Plus size={15} /> New crew
          </button>
          <button
            role="tab"
            aria-selected={mode === 'join'}
            className={mode === 'join' ? 'active' : ''}
            onClick={() => setMode('join')}
          >
            <Ticket size={15} /> I have a code
          </button>
        </div>
        <form
          onSubmit={(event) => {
            event.preventDefault();
            setError('');
            submit.mutate();
          }}
        >
          <fieldset disabled={submit.isPending}>
            <label>
              Your name in the crew
              <input
                value={displayName}
                onChange={(e) => setDisplayName(e.target.value)}
                maxLength={80}
                required
              />
            </label>
            {mode === 'create' ? (
              <>
                <label>
                  Crew name
                  <input
                    value={crewName}
                    onChange={(e) => setCrewName(e.target.value)}
                    maxLength={80}
                    placeholder="Flat 4B"
                    required
                    autoFocus
                  />
                </label>
                <label>
                  Crew time zone
                  <select value={timezone} onChange={(e) => setTimezone(e.target.value)}>
                    {[...new Set([timezone, ...Intl.supportedValuesOf('timeZone')])].map((zone) => (
                      <option value={zone} key={zone}>
                        {zoneName(zone)} · {zone}
                      </option>
                    ))}
                  </select>
                </label>
              </>
            ) : (
              <label>
                Invite code
                <input
                  value={code}
                  onChange={(e) => setCode(e.target.value)}
                  placeholder="K7QX-M2PA"
                  autoComplete="off"
                  required
                  autoFocus
                />
              </label>
            )}
            {error && (
              <div className="notice error" role="alert">
                {error}
              </div>
            )}
            <button className="primary" type="submit">
              {submit.isPending ? 'One sec…' : mode === 'create' ? 'Create crew' : 'Join crew'}
              <ArrowRight size={16} />
            </button>
          </fieldset>
        </form>
        <button className="text-button" onClick={onSignOut}>
          <LogOut size={14} /> Use another account
        </button>
      </div>
    </div>
  );
}
