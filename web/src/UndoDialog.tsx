import { useEffect, useRef } from 'react';
import { completedLabel } from './time';
import type { Occurrence, Person } from './types';
export function sameCompletion(shown: Occurrence, current: Occurrence | undefined) {
  return Boolean(
    current &&
    current.status === 'done' &&
    current.revision === shown.revision &&
    current.completed_by === shown.completed_by &&
    current.completed_at === shown.completed_at,
  );
}
export default function UndoDialog({
  item,
  people,
  zone,
  busy,
  disabled,
  onClose,
  onConfirm,
}: {
  item: Occurrence;
  people: Person[];
  zone: string;
  busy: boolean;
  disabled: boolean;
  onClose: () => void;
  onConfirm: () => void;
}) {
  const panel = useRef<HTMLElement>(null);
  const safe = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    const previous = document.activeElement as HTMLElement | null;
    safe.current?.focus();
    return () => previous?.focus();
  }, []);
  const author = people.find((p) => p.account === item.completed_by)?.name || 'someone else';
  return (
    <div
      className="modal-backdrop"
      onClick={(event) => {
        if (event.target === event.currentTarget && !busy) onClose();
      }}
    >
      <section
        ref={panel}
        className="editor undo-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="undo-title"
        aria-describedby="undo-description"
        onKeyDown={(event) => {
          if (event.key === 'Escape' && !busy) {
            event.preventDefault();
            onClose();
          }
          if (event.key === 'Tab') {
            const buttons = Array.from(
              panel.current!.querySelectorAll<HTMLButtonElement>('button:not(:disabled)'),
            );
            const first = buttons[0],
              last = buttons.at(-1);
            if (!buttons.length) event.preventDefault();
            else if (event.shiftKey && document.activeElement === first) {
              event.preventDefault();
              last?.focus();
            } else if (!event.shiftKey && document.activeElement === last) {
              event.preventDefault();
              first?.focus();
            }
          }
        }}
      >
        <span className="eyebrow">TEAMWORK</span>
        <h2 id="undo-title">Undo {author}'s check-off?</h2>
        <p id="undo-description">
          {author} checked this off at{' '}
          {item.completed_at && completedLabel(item.completed_at, zone)}. Sure you want to undo it?
        </p>
        <strong>
          {item.routine.name} · {item.routine.assignee}
        </strong>
        <div className="editor-actions">
          <button ref={safe} className="secondary" onClick={onClose} disabled={busy}>
            Keep it
          </button>
          <button className="primary" onClick={onConfirm} disabled={busy || disabled}>
            {busy ? 'Saving…' : 'Undo'}
          </button>
        </div>
      </section>
    </div>
  );
}
