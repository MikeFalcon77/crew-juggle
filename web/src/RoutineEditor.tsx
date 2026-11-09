import { useState } from 'react';
import { X, CalendarDays, ArrowRight, Link2 } from 'lucide-react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { api } from './api';
import PersonAvatar from './PersonAvatar';
import { useDialogKeyboard } from './useDialogKeyboard';
import { WEEKDAYS, daysLabel } from './time';
import type { AvatarMeta, Person, Routine, Version } from './types';

export function nextDate(date: string) {
  const day = new Date(`${date}T12:00:00Z`);
  day.setUTCDate(day.getUTCDate() + 1);
  return day.toISOString().slice(0, 10);
}

/** A readable, URL-safe id from the name, unique enough inside one crew. */
export function routineId(name: string) {
  const slug = name
    .normalize('NFKD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 40);
  return `${slug || 'routine'}-${crypto.randomUUID().slice(0, 6)}`;
}

export default function RoutineEditor({
  version,
  versions,
  people,
  avatars,
  initialPersonId,
  today,
  disabled,
  onClose,
  onError,
  onSaved,
}: {
  version: Version | null;
  versions: Version[];
  people: Person[];
  avatars?: Record<string, AvatarMeta>;
  initialPersonId: string;
  today: string;
  disabled: boolean;
  onClose: () => void;
  onError: (message: string) => void;
  onSaved?: (effective: string) => void;
}) {
  const client = useQueryClient();
  // Keep the revision of the form the user actually opened: a background
  // refresh must never grant permission to overwrite a newer edit.
  const [expectedRevision] = useState(version?.routine.revision || 0);
  const initialPerson = people.find((p) => p.id === initialPersonId) || people[0];
  const [form, setForm] = useState<Routine>(
    version?.routine || {
      id: '',
      revision: 0,
      assignee: initialPerson.name,
      person_id: initialPerson.id,
      name: '',
      note: '',
      group: '',
      time: '08:00',
      days: [],
      start_date: today,
      end_date: null,
      enabled: true,
      prerequisite: null,
    },
  );
  const [effective, setEffective] = useState(
    version ? [nextDate(today), version.effective_from].sort().at(-1)! : today,
  );
  const [message, setMessage] = useState('');
  const assignee = people.find((person) => person.id === form.person_id);
  const isSecondStep = Boolean(form.prerequisite);
  const hasSecondStep = versions.some((v) => v.routine.prerequisite === form.id);
  const firstStepChoices = versions.filter(
    (v) =>
      !v.routine.prerequisite &&
      v.routine.enabled &&
      !versions.some((other) => other.routine.prerequisite === v.routine.id),
  );
  const change = <K extends keyof Routine>(key: K, value: Routine[K]) =>
    setForm((current) => ({ ...current, [key]: value }));
  const toggleDay = (day: number) =>
    change(
      'days',
      form.days.includes(day) ? form.days.filter((d) => d !== day) : [...form.days, day].sort(),
    );
  const save = useMutation({
    mutationFn: () =>
      api('/schedule', 'PUT', {
        operation_id: crypto.randomUUID(),
        expected_revision: expectedRevision,
        effective_from: effective,
        routine: { ...form, id: form.id || routineId(form.name) },
      }),
    onSuccess: async () => {
      await client.invalidateQueries({ queryKey: ['schedule'] });
      await client.invalidateQueries({ queryKey: ['day'] });
      onSaved?.(effective);
      onClose();
    },
    onError: (error) => {
      setMessage(error.message);
      onError(error.message);
      void client.invalidateQueries({ queryKey: ['schedule'] });
    },
  });
  const dialogRef = useDialogKeyboard(save.isPending, onClose);
  return (
    <div
      className="modal-backdrop"
      onClick={(event) => {
        if (event.target === event.currentTarget && !save.isPending) onClose();
      }}
    >
      <section
        ref={dialogRef}
        className="editor"
        role="dialog"
        aria-modal="true"
        aria-labelledby="editor-title"
      >
        <div className="editor-head">
          <div>
            <span className="eyebrow">ROUTINES</span>
            <h2 id="editor-title">{version ? 'Edit routine' : 'New routine'}</h2>
          </div>
          <button
            className="icon-button"
            aria-label="Close"
            onClick={onClose}
            disabled={save.isPending}
          >
            <X size={20} />
          </button>
        </div>
        <form
          onSubmit={(event) => {
            event.preventDefault();
            if (!save.isPending && !disabled && navigator.onLine) save.mutate();
          }}
        >
          <fieldset disabled={save.isPending}>
            {assignee && (
              <div className={`recipient-preview tone-${assignee.tone}`}>
                <PersonAvatar person={assignee} meta={avatars?.[assignee.id]} />
                <div>
                  <small>Who does it</small>
                  <strong>{assignee.name}</strong>
                </div>
              </div>
            )}
            <label>
              What
              <input
                value={form.name}
                onChange={(event) => change('name', event.target.value)}
                maxLength={120}
                placeholder="Take out the trash"
                required
                autoFocus
              />
            </label>
            {!version && firstStepChoices.length > 0 && (
              <label>
                Second step of
                <select
                  value={form.prerequisite || ''}
                  onChange={(event) => change('prerequisite', event.target.value || null)}
                >
                  <option value="">Nothing, it stands on its own</option>
                  {firstStepChoices.map((v) => (
                    <option key={v.routine.id} value={v.routine.id}>
                      After “{v.routine.name}”
                    </option>
                  ))}
                </select>
              </label>
            )}
            {!isSecondStep && (
              <div className="form-grid">
                <label>
                  Who
                  <select
                    value={form.person_id}
                    onChange={(event) => {
                      const person = people.find((p) => p.id === event.target.value)!;
                      setForm((current) => ({
                        ...current,
                        person_id: person.id,
                        assignee: person.name,
                      }));
                    }}
                  >
                    {people.map((person) => (
                      <option key={person.id} value={person.id}>
                        {person.name}
                      </option>
                    ))}
                  </select>
                </label>
                <label>
                  Time
                  <input
                    type="time"
                    value={form.time}
                    onChange={(event) => change('time', event.target.value)}
                    required
                  />
                </label>
              </div>
            )}
            {!isSecondStep && (
              <fieldset className="weekday-picker">
                <legend>
                  Days <span>{daysLabel(form.days)}</span>
                </legend>
                <div>
                  {WEEKDAYS.map(({ day, short, long }) => (
                    <button
                      type="button"
                      key={day}
                      className={form.days.includes(day) ? 'active' : ''}
                      aria-pressed={form.days.includes(day)}
                      aria-label={long}
                      onClick={() => toggleDay(day)}
                    >
                      {short}
                    </button>
                  ))}
                </div>
                <small>No days picked means every day.</small>
              </fieldset>
            )}
            <label>
              Note
              <textarea
                value={form.note}
                onChange={(event) => change('note', event.target.value)}
                placeholder="Robot parts are not dishes."
                maxLength={1000}
                rows={2}
              />
            </label>
            {!isSecondStep && (
              <label>
                Label
                <input
                  value={form.group}
                  onChange={(event) => change('group', event.target.value)}
                  maxLength={60}
                  placeholder="Kitchen, Laundry, Pets…"
                  list="routine-groups"
                />
                <datalist id="routine-groups">
                  {[...new Set(versions.map((v) => v.routine.group).filter(Boolean))].map(
                    (group) => (
                      <option key={group}>{group}</option>
                    ),
                  )}
                </datalist>
              </label>
            )}
            <div className="form-grid">
              <label>
                Changes start on
                <input
                  type="date"
                  value={effective}
                  min={today}
                  onChange={(event) => setEffective(event.target.value)}
                  required
                />
              </label>
              {!isSecondStep && (
                <label>
                  Last day (optional)
                  <input
                    type="date"
                    value={form.end_date || ''}
                    min={form.start_date}
                    onChange={(event) => change('end_date', event.target.value || null)}
                  />
                </label>
              )}
            </div>
            {!isSecondStep && (
              <label className="checkbox-label">
                <input
                  type="checkbox"
                  checked={form.enabled}
                  onChange={(event) => change('enabled', event.target.checked)}
                />
                Routine is on
              </label>
            )}
            <div className="editor-note">
              {isSecondStep || hasSecondStep ? <Link2 size={17} /> : <CalendarDays size={17} />}
              <span>
                {isSecondStep
                  ? 'A second step shares who, when and which days with its first step. Edit the first step to move both.'
                  : 'Done tasks stay in history exactly as they were. Turning a routine off removes its unfinished tasks from the start date.'}
                {hasSecondStep && ' Who, when and which days apply to both steps.'}
              </span>
            </div>
            {message && (
              <div className="notice error" role="alert">
                {message}
              </div>
            )}
            <div className="editor-actions">
              <button type="button" className="secondary" onClick={onClose}>
                Cancel
              </button>
              <button className="primary" type="submit" disabled={disabled || !navigator.onLine}>
                {save.isPending ? 'Saving…' : 'Save'}
                <ArrowRight size={16} />
              </button>
            </div>
          </fieldset>
        </form>
      </section>
    </div>
  );
}
