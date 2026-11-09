import { useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { Globe2, Check } from 'lucide-react';
import { api } from './api';
import { browserZone as deviceZone, getCrewZone, zoneName } from './time';
import type { CrewPreferences } from './types';
export default function SettingsView({
  data,
  myUid,
  disabled,
  onError,
}: {
  data: CrewPreferences;
  myUid: string;
  disabled: boolean;
  onError: (message: string) => void;
}) {
  const client = useQueryClient();
  const browserZone = deviceZone();
  const crewZone = getCrewZone();
  const [edit, setEdit] = useState<{
    timezone: string;
    revision: number;
    operationId: string;
  } | null>(null);
  const [saved, setSaved] = useState(false);
  const zones = [
    ...new Set([
      crewZone,
      browserZone,
      data.preferences.timezone,
      ...Intl.supportedValuesOf('timeZone'),
    ]),
  ];
  const selected = edit?.timezone || data.preferences.timezone;
  function choose(timezone: string) {
    setSaved(false);
    setEdit({ timezone, revision: data.preferences.revision, operationId: crypto.randomUUID() });
  }
  const save = useMutation({
    mutationFn: () =>
      api('/me/preferences', 'PUT', {
        operation_id: edit?.operationId || crypto.randomUUID(),
        expected_revision: edit?.revision ?? data.preferences.revision,
        timezone: selected,
      }),
    onSuccess: async () => {
      await client.invalidateQueries({ queryKey: ['preferences'] });
      setEdit(null);
      setSaved(true);
    },
    onError: (error) => {
      onError(error.message);
      setEdit(null);
      void client.invalidateQueries({ queryKey: ['preferences'] });
    },
  });
  return (
    <>
      <div className="section-heading">
        <div>
          <span className="eyebrow">JUST FOR YOU</span>
          <h1>Your time zone</h1>
          <p>
            The crew's calendar runs on {zoneName(crewZone)} time. Traveling? Pick your zone and
            times also show up in your local clock.
          </p>
        </div>
        <Globe2 size={30} />
      </div>
      <div className="settings-grid">
        <section className="settings-card">
          <h2>Show times for</h2>
          <p>
            This device is on <strong>{zoneName(browserZone)}</strong> time. Your choice syncs to
            all your devices.
          </p>
          <div className="zone-shortcuts">
            <button
              className="secondary"
              onClick={() => choose(crewZone)}
              disabled={disabled || save.isPending}
            >
              Crew time · {zoneName(crewZone)}
            </button>
            <button
              className="text-button"
              onClick={() => choose(browserZone)}
              disabled={disabled || save.isPending}
            >
              Use this device's zone
            </button>
          </div>
          <form
            onSubmit={(event) => {
              event.preventDefault();
              save.mutate();
            }}
          >
            <label>
              Your time zone
              <select
                value={selected}
                onChange={(event) => choose(event.target.value)}
                disabled={disabled || save.isPending}
              >
                {zones.map((zone) => (
                  <option value={zone} key={zone}>
                    {zoneName(zone)} · {zone}
                  </option>
                ))}
              </select>
            </label>
            <p>Routines and reminders keep their crew times; only the labels change.</p>
            <button
              className="primary"
              disabled={disabled || save.isPending || (!edit && data.preferences.revision > 0)}
            >
              {save.isPending ? 'Saving…' : 'Save time zone'}
            </button>
            {saved && (
              <span className="settings-saved" role="status">
                <Check size={16} />
                Saved
              </span>
            )}
          </form>
        </section>
        <section className="settings-card">
          <h2>Where the crew is</h2>
          {data.members.map((member) => (
            <div className="member-zone" key={member.uid}>
              <strong>
                {member.name}
                {member.uid === myUid ? ' · you' : ''}
              </strong>
              <span>
                {zoneName(member.preferences.timezone)}
                {member.preferences.revision === 0 ? ' · crew time' : ''}
              </span>
            </div>
          ))}
          <p>Everyone picks only their own zone.</p>
        </section>
      </div>
    </>
  );
}
