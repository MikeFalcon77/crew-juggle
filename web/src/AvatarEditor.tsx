import { useDialogKeyboard } from './useDialogKeyboard';
import { useEffect, useRef, useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { api, ApiError } from './api';
import { cropLayout } from './crop';
import type { Person, AvatarMeta } from './types';
export default function AvatarEditor({
  person,
  meta,
  disabled,
  onClose,
  onError,
}: {
  person: Person;
  meta?: AvatarMeta;
  disabled: boolean;
  onClose: () => void;
  onError: (message: string) => void;
}) {
  const [expectedRevision] = useState(meta?.revision || 0);
  const client = useQueryClient();
  const canvas = useRef<HTMLCanvasElement>(null);
  const drag = useRef<{ x: number; y: number } | null>(null);
  const [source, setSource] = useState('');
  const [image, setImage] = useState<HTMLImageElement | null>(null);
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [operation, setOperation] = useState(crypto.randomUUID());
  useEffect(() => {
    if (!source) return;
    let alive = true;
    const img = new Image();
    img.onload = () => {
      if (alive) {
        setImage(img);
        setLoading(false);
      }
    };
    img.onerror = () => {
      if (alive) {
        setError("Couldn't open the photo. Pick a JPEG, PNG or WebP.");
        setLoading(false);
      }
    };
    img.src = source;
    return () => {
      alive = false;
      URL.revokeObjectURL(source);
    };
  }, [source]);
  useEffect(() => {
    if (!image || !canvas.current) return;
    const box = cropLayout(image.naturalWidth, image.naturalHeight, zoom, pan.x, pan.y);
    const context = canvas.current.getContext('2d')!;
    context.clearRect(0, 0, 256, 256);
    context.drawImage(
      image,
      (256 - box.width) / 2 + box.x,
      (256 - box.height) / 2 + box.y,
      box.width,
      box.height,
    );
  }, [image, zoom, pan]);
  function move(x: number, y: number) {
    if (!image) return;
    const box = cropLayout(image.naturalWidth, image.naturalHeight, zoom, x, y);
    setPan({ x: box.x, y: box.y });
    setOperation(crypto.randomUUID());
  }
  async function invalidate() {
    await Promise.all(
      ['people', 'day', 'schedule', 'avatar'].map((key) =>
        client.invalidateQueries({ queryKey: [key] }),
      ),
    );
    onClose();
  }
  const save = useMutation({
    mutationFn: async ({ remove, operationId }: { remove: boolean; operationId: string }) => {
      if (remove)
        return api(`/people/${person.id}/avatar`, 'DELETE', {
          operation_id: operationId,
          expected_revision: expectedRevision,
        });
      const file = await new Promise<Blob>((resolve, reject) =>
        canvas.current!.toBlob(
          (blob) => (blob ? resolve(blob) : reject(new Error("Couldn't prepare the photo"))),
          'image/png',
        ),
      );
      const form = new FormData();
      form.set('operation_id', operationId);
      form.set('expected_revision', String(expectedRevision));
      form.set('file', file, 'avatar.png');
      return api(`/people/${person.id}/avatar`, 'PUT', form);
    },
    onSuccess: invalidate,
    onError: (e) => {
      setError(e.message);
      onError(e.message);
      void client.invalidateQueries({ queryKey: ['people'] });
      if (e instanceof ApiError && e.status === 409) onClose();
    },
  });
  const panel = useDialogKeyboard(save.isPending, onClose);
  return (
    <div className="modal-backdrop">
      <section
        ref={panel}
        className="editor avatar-editor"
        role="dialog"
        aria-modal="true"
        aria-labelledby="avatar-title"
      >
        <span className="eyebrow">PROFILE PHOTO</span>
        <h2 id="avatar-title">Photo · {person.name}</h2>
        <label className="photo-file">
          Pick a photo
          <input
            type="file"
            accept="image/jpeg,image/png,image/webp"
            disabled={disabled || save.isPending}
            onChange={(event) => {
              const file = event.target.files?.[0];
              if (!file) return;
              setError('');
              event.target.value = '';
              if (
                !['image/jpeg', 'image/png', 'image/webp'].includes(file.type) ||
                file.size > 10 * 1024 * 1024
              ) {
                setError('Pick a JPEG, PNG or WebP up to 10 MB.');
                return;
              }
              setImage(null);
              setLoading(true);
              setZoom(1);
              setPan({ x: 0, y: 0 });
              setSource(URL.createObjectURL(file));
              setOperation(crypto.randomUUID());
            }}
          />
        </label>
        {loading && <p role="status">Opening photo…</p>}
        {source && image && (
          <>
            <p>Drag and zoom to frame it.</p>
            <canvas
              ref={canvas}
              width={256}
              height={256}
              tabIndex={0}
              className="crop-stage"
              role="img"
              aria-label="Avatar frame. Use arrow keys to move it"
              onPointerDown={(event) => {
                if (save.isPending) return;
                event.currentTarget.setPointerCapture(event.pointerId);
                drag.current = { x: event.clientX, y: event.clientY };
              }}
              onPointerMove={(event) => {
                if (!drag.current || save.isPending) return;
                const factor = 256 / event.currentTarget.getBoundingClientRect().width;
                move(
                  pan.x + (event.clientX - drag.current.x) * factor,
                  pan.y + (event.clientY - drag.current.y) * factor,
                );
                drag.current = { x: event.clientX, y: event.clientY };
              }}
              onPointerUp={() => {
                drag.current = null;
              }}
              onPointerCancel={() => {
                drag.current = null;
              }}
              onKeyDown={(event) => {
                const keys: Record<string, [number, number]> = {
                  ArrowLeft: [-8, 0],
                  ArrowRight: [8, 0],
                  ArrowUp: [0, -8],
                  ArrowDown: [0, 8],
                };
                if (keys[event.key] && !save.isPending) {
                  event.preventDefault();
                  move(pan.x + keys[event.key][0], pan.y + keys[event.key][1]);
                }
              }}
            />
            <label>
              Zoom
              <input
                type="range"
                min="1"
                max="3"
                step="0.01"
                value={zoom}
                disabled={save.isPending}
                onChange={(event) => {
                  setZoom(Number(event.target.value));
                  setPan({ x: 0, y: 0 });
                  setOperation(crypto.randomUUID());
                }}
              />
            </label>
          </>
        )}
        {error && (
          <div className="notice error" role="alert">
            {error}
          </div>
        )}
        <div className="editor-actions">
          {meta?.has_photo && (
            <button
              className="text-button"
              disabled={disabled || save.isPending}
              onClick={() => {
                save.mutate({ remove: true, operationId: crypto.randomUUID() });
              }}
            >
              Remove photo
            </button>
          )}
          <button className="secondary" autoFocus disabled={save.isPending} onClick={onClose}>
            Cancel
          </button>
          <button
            className="primary"
            disabled={disabled || save.isPending || !image}
            onClick={() => save.mutate({ remove: false, operationId: operation })}
          >
            {save.isPending ? 'Saving…' : 'Save photo'}
          </button>
        </div>
      </section>
    </div>
  );
}
