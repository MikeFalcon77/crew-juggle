import { useEffect, useRef } from 'react';
export function useDialogKeyboard(busy: boolean, onClose: () => void) {
  const ref = useRef<HTMLElement>(null);
  const current = useRef({ busy, onClose });
  current.current = { busy, onClose };
  useEffect(() => {
    const previous = document.activeElement as HTMLElement | null;
    const panel = ref.current;
    if (!panel) return;
    function key(event: KeyboardEvent) {
      if (event.key === 'Escape' && !current.current.busy) {
        event.preventDefault();
        current.current.onClose();
      }
      if (event.key === 'Tab') {
        const fields = Array.from(
          panel!.querySelectorAll<HTMLElement>(
            'button:not(:disabled),input:not(:disabled),textarea:not(:disabled),select:not(:disabled),[tabindex="0"]',
          ),
        ).filter((el) => el.getClientRects().length > 0);
        const first = fields[0],
          last = fields.at(-1);
        if (!fields.length) event.preventDefault();
        else if (event.shiftKey && document.activeElement === first) {
          event.preventDefault();
          last?.focus();
        } else if (!event.shiftKey && document.activeElement === last) {
          event.preventDefault();
          first?.focus();
        }
      }
    }
    panel.addEventListener('keydown', key);
    return () => {
      panel.removeEventListener('keydown', key);
      previous?.focus();
    };
  }, []);
  return ref;
}
