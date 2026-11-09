import { useQuery } from '@tanstack/react-query';
import { X } from 'lucide-react';
import { api } from './api';
import RoutineEditor from './RoutineEditor';
import type { Schedule } from './types';

export type RoutineEditRequest =
  { kind: 'new'; personId?: string } | { kind: 'edit'; routineId: string };

/** Loads the latest version before editing: today's tasks are frozen snapshots. */
export default function RoutineEditorLoader({
  request,
  disabled,
  onClose,
  onError,
  onSaved,
}: {
  request: RoutineEditRequest;
  disabled: boolean;
  onClose: () => void;
  onError: (message: string) => void;
  onSaved: (effective: string) => void;
}) {
  const query = useQuery({
    queryKey: ['schedule', 'editor'],
    queryFn: () => api<Schedule>('/schedule'),
    gcTime: 0,
    staleTime: Infinity,
    retry: false,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
  });
  const data = query.data;
  const version =
    request.kind === 'edit' ? data?.versions.find((v) => v.routine.id === request.routineId) : null;
  const problem =
    query.isError && !data
      ? "Couldn't load routines. Try again."
      : data && !data.people.length
        ? 'Add someone to your crew first.'
        : data && request.kind === 'edit' && !version
          ? 'This routine is gone. Refresh the list.'
          : null;
  if (query.isPending || problem)
    return (
      <div className="modal-backdrop">
        <section className="editor" role="dialog" aria-modal="true" aria-label="Routine">
          <div className="editor-head">
            <h2>{request.kind === 'new' ? 'New routine' : 'Edit routine'}</h2>
            <button className="icon-button" aria-label="Close" onClick={onClose}>
              <X size={20} />
            </button>
          </div>
          <p role={problem ? 'alert' : 'status'}>{problem || 'Loading the latest version…'}</p>
          {query.isError && (
            <button
              className="secondary"
              disabled={disabled || query.isFetching}
              onClick={() => void query.refetch()}
            >
              Try again
            </button>
          )}
        </section>
      </div>
    );
  return (
    <RoutineEditor
      version={version || null}
      versions={data!.versions}
      people={data!.people}
      avatars={data!.avatars}
      initialPersonId={request.kind === 'new' ? request.personId || '' : ''}
      today={data!.today}
      disabled={disabled}
      onClose={onClose}
      onError={onError}
      onSaved={onSaved}
    />
  );
}
