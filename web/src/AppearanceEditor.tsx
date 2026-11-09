import { useDialogKeyboard } from './useDialogKeyboard';
import { useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { api } from './api';
import RoutineFace, { moods } from './RoutineFace';
import type { Appearance, Mood, Routine } from './types';
export default function AppearanceEditor({
  routine,
  appearance,
  disabled,
  onClose,
  onError,
}: {
  routine: Routine;
  appearance?: Appearance;
  disabled: boolean;
  onClose: () => void;
  onError: (message: string) => void;
}) {
  const [expectedRevision] = useState(appearance?.revision || 0);
  const client = useQueryClient();
  const [mood, setMood] = useState<Mood>(appearance?.mood || 'happy');
  const [operation, setOperation] = useState(crypto.randomUUID());
  const save = useMutation({
    mutationFn: () =>
      api(`/schedule/${routine.id}/appearance`, 'PUT', {
        operation_id: operation,
        expected_revision: expectedRevision,
        mood,
      }),
    onSuccess: async () => {
      await Promise.all([
        client.invalidateQueries({ queryKey: ['schedule'] }),
        client.invalidateQueries({ queryKey: ['day'] }),
      ]);
      onClose();
    },
    onError: (error) => {
      onError(error.message);
      void client.invalidateQueries({ queryKey: ['schedule'] });
      onClose();
    },
  });
  const panel = useDialogKeyboard(save.isPending, onClose);
  return (
    <div className="modal-backdrop">
      <section
        ref={panel}
        className="editor appearance-editor"
        role="dialog"
        aria-modal="true"
        aria-labelledby="appearance-title"
      >
        <span className="eyebrow">ROUTINE MOOD</span>
        <h2 id="appearance-title">{routine.name}</h2>
        <p>Pick a mood. The whole crew sees it right away.</p>
        <div className="mood-picker">
          {moods.map((item) => (
            <button
              type="button"
              key={item.id}
              aria-pressed={mood === item.id}
              className={`mood-choice ${mood === item.id ? 'selected' : ''}`}
              disabled={save.isPending}
              onClick={() => {
                setMood(item.id);
                setOperation(crypto.randomUUID());
              }}
            >
              <RoutineFace mood={item.id} />
              <span>{item.label}</span>
            </button>
          ))}
        </div>
        <div className="editor-actions">
          <button className="secondary" autoFocus disabled={save.isPending} onClick={onClose}>
            Cancel
          </button>
          <button
            className="primary"
            disabled={disabled || save.isPending}
            onClick={() => save.mutate()}
          >
            {save.isPending ? 'Saving…' : 'Save mood'}
          </button>
        </div>
      </section>
    </div>
  );
}
