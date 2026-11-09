import { useEffect, useRef, useState } from 'react';
import { Camera, ImagePlus, LoaderCircle, Trash2 } from 'lucide-react';
import { Modal } from './ShoppingComponents';
import './ShoppingCamera.css';

export function ShoppingCamera({
  onClose,
  onSubmit,
  offline,
}: {
  onClose: () => void;
  onSubmit: (photos: string[]) => Promise<void>;
  offline: boolean;
}) {
  const video = useRef<HTMLVideoElement>(null);
  const gallery = useRef<HTMLInputElement>(null);
  const nativeCamera = useRef<HTMLInputElement>(null);
  const stream = useRef<MediaStream | null>(null);
  const generation = useRef(0);
  const lock = useRef(false);
  const urls = useRef<string[]>([]);
  const [ready, setReady] = useState(false);
  const [cameraError, setCameraError] = useState('');
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [long, setLong] = useState(false);
  const [photos, setPhotos] = useState<string[]>([]);
  const [retry, setRetry] = useState<string[]>([]);

  function stop() {
    generation.current++;
    stream.current?.getTracks().forEach((track) => track.stop());
    stream.current = null;
    if (video.current) video.current.srcObject = null;
  }
  useEffect(() => {
    let active = true;
    async function start() {
      const request = ++generation.current;
      setReady(false);
      if (!navigator.mediaDevices?.getUserMedia) {
        setCameraError('No camera in this browser. Pick a photo or open the system camera.');
        return;
      }
      try {
        const next = await navigator.mediaDevices.getUserMedia({
          audio: false,
          video: {
            facingMode: { ideal: 'environment' },
            width: { ideal: 2560 },
            height: { ideal: 1920 },
          },
        });
        if (!active || request !== generation.current) {
          next.getTracks().forEach((track) => track.stop());
          return;
        }
        stream.current = next;
        if (video.current) {
          video.current.srcObject = next;
          await video.current.play();
          if (active && request === generation.current) {
            setReady(true);
            setCameraError('');
          }
        }
      } catch {
        if (active && request === generation.current) {
          stop();
          setCameraError('No camera access. Allow it in browser settings or pick a photo.');
        }
      }
    }
    const visibility = () => {
      if (document.hidden) {
        stop();
        setReady(false);
      } else if (!lock.current) void start();
    };
    void start();
    document.addEventListener('visibilitychange', visibility);
    window.addEventListener('pagehide', stop);
    return () => {
      active = false;
      stop();
      document.removeEventListener('visibilitychange', visibility);
      window.removeEventListener('pagehide', stop);
      urls.current.forEach((url) => URL.revokeObjectURL(url));
    };
  }, []);

  async function submit(batch: string[]) {
    stop();
    setReady(false);
    setRetry(batch);
    await onSubmit(batch);
    setRetry([]);
  }
  async function accept(files: Blob[]) {
    if (!files.length || lock.current) return;
    lock.current = true;
    setBusy(true);
    setError('');
    const accepted: string[] = [];
    try {
      if (files.length + (long ? photos.length : 0) > 5)
        throw new Error('Up to five photos of one receipt');
      for (const file of files) {
        if (
          file.size > 10 * 1024 * 1024 ||
          !['image/jpeg', 'image/png', 'image/webp'].includes(file.type)
        )
          throw new Error('JPEG, PNG or WebP up to 10 MB');
        const url = URL.createObjectURL(file);
        urls.current.push(url);
        const image = new Image();
        image.src = url;
        await image.decode();
        accepted.push(url);
      }
      if (long) setPhotos((previous) => [...previous, ...accepted]);
      else await submit(accepted);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Couldn't save the photo");
    } finally {
      lock.current = false;
      setBusy(false);
    }
  }
  async function shutter() {
    if (lock.current || !ready || !video.current || (long && photos.length >= 5)) return;
    const canvas = document.createElement('canvas');
    canvas.width = video.current.videoWidth;
    canvas.height = video.current.videoHeight;
    canvas.getContext('2d')?.drawImage(video.current, 0, 0);
    lock.current = true;
    setBusy(true);
    try {
      const blob = await new Promise<Blob | null>((resolve) =>
        canvas.toBlob(resolve, 'image/jpeg', 0.94),
      );
      lock.current = false;
      if (!blob) throw new Error("Couldn't take the picture. Try again.");
      await accept([blob]);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Couldn't take the picture");
    } finally {
      lock.current = false;
      setBusy(false);
    }
  }
  async function finish() {
    if (lock.current) return;
    lock.current = true;
    setBusy(true);
    setError('');
    try {
      await submit(retry.length ? retry : photos);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Couldn't save the photo");
    } finally {
      lock.current = false;
      setBusy(false);
    }
  }
  return (
    <Modal
      title="Scan a receipt"
      full
      onClose={() => {
        if (!lock.current) onClose();
      }}
    >
      <div className="sp-camera-view">
        <video ref={video} autoPlay muted playsInline aria-label="Receipt camera" />
        <div className="sp-camera-guide" aria-hidden="true" />
        <div className="sp-camera-top">
          <button
            className={long ? 'active' : ''}
            disabled={busy || photos.length > 0 || retry.length > 0}
            aria-pressed={long}
            onClick={() => setLong(!long)}
          >
            Long receipt{long ? ` · ${photos.length}/5` : ''}
          </button>
          <p>
            {long
              ? 'Shoot long receipts top to bottom with a little overlap'
              : 'Whole receipt in frame? It starts scanning right away'}
          </p>
        </div>
        <div className="sp-camera-bottom">
          {cameraError && <p role="status">{cameraError}</p>}
          {error && <p role="alert">{error}</p>}
          {long && photos.length > 0 && (
            <div className="sp-camera-thumbnails">
              {photos.map((url, index) => (
                <div key={url}>
                  <img src={url} alt={`Receipt part ${index + 1}`} />
                  <button
                    disabled={busy}
                    aria-label={`Remove part ${index + 1}`}
                    onClick={() => {
                      setPhotos((all) => all.filter((p) => p !== url));
                      URL.revokeObjectURL(url);
                    }}
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              ))}
            </div>
          )}
          {!ready && !busy && !retry.length && (
            <button className="sp-camera-native" onClick={() => nativeCamera.current?.click()}>
              <Camera size={18} /> Open system camera
            </button>
          )}
          {retry.length > 0 && !busy && (
            <button className="sp-camera-done" onClick={() => void finish()}>
              Retry upload
            </button>
          )}
          {long && photos.length > 0 && !retry.length && (
            <button className="sp-camera-done" disabled={busy} onClick={() => void finish()}>
              Done · {photos.length} photos
            </button>
          )}
          <div className="sp-camera-controls">
            <span aria-live="polite">
              {busy
                ? 'Saving…'
                : offline
                  ? 'Offline · saved on this device'
                  : 'Any receipt · no list needed'}
            </span>
            <button
              className="sp-camera-shutter"
              aria-label="Take photo"
              disabled={!ready || busy || retry.length > 0 || (long && photos.length >= 5)}
              onClick={() => void shutter()}
            >
              {busy ? <LoaderCircle className="sp-spin" size={30} /> : <span />}
            </button>
            <button
              className="sp-camera-gallery"
              disabled={busy || retry.length > 0}
              onClick={() => gallery.current?.click()}
              aria-label="Pick from gallery"
            >
              <ImagePlus size={28} />
              <span>Gallery</span>
            </button>
          </div>
        </div>
        <input
          hidden
          data-testid="receipt-gallery"
          ref={gallery}
          type="file"
          multiple
          accept="image/jpeg,image/png,image/webp"
          onChange={(e) => {
            void accept(Array.from(e.target.files || []));
            e.target.value = '';
          }}
        />
        <input
          hidden
          data-testid="receipt-native-camera"
          ref={nativeCamera}
          type="file"
          capture="environment"
          accept="image/jpeg,image/png,image/webp"
          onChange={(e) => {
            void accept(Array.from(e.target.files || []));
            e.target.value = '';
          }}
        />
      </div>
    </Modal>
  );
}
