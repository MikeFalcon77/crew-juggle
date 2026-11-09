import { useCallback, useEffect, useRef, useState, type SetStateAction } from 'react';
import { apiAt, ApiError } from './api';
import { currentUserId } from './session';
import {
  crewDate,
  type ListItem,
  type Product,
  type Receipt,
  type CorrectionRule,
} from './shoppingModels';
import type { Me } from './types';

export type ShoppingSnapshot = {
  rules?: CorrectionRule[];
  revision: number;
  today: string;
  list: ListItem[];
  catalog: Product[];
  receipts: Receipt[];
  purchases: Receipt[];
  imports: { id: string; text: string; status: string; error?: string }[];
  aiConfigured: boolean;
};
type Action =
  | { kind: 'add'; items: { id: string; name: string; quantity: string }[] }
  | { kind: 'mark'; id: string; revision: number; done: boolean }
  | { kind: 'remove'; id: string; revision: number }
  | { kind: 'product'; product: Product }
  | { kind: 'import'; id: string; text: string };
export type PendingShopping =
  | {
      id: string;
      kind: 'command';
      command: { operation_id: string; action: Action };
      blocked?: string;
    }
  | { id: string; kind: 'upload'; photos: Blob[]; createdAt: string; blocked?: string };
type Account = { uid: string; snapshot: ShoppingSnapshot; queue: PendingShopping[]; me?: Me };
const EMPTY = (): ShoppingSnapshot => ({
  revision: 0,
  today: crewDate(),
  list: [],
  catalog: [],
  receipts: [],
  purchases: [],
  imports: [],
  aiConfigured: false,
});
const DB_NAME = 'crew-juggle-shopping-v1';
let database: Promise<IDBDatabase> | undefined;
function db() {
  return (database ||= new Promise<IDBDatabase>((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, 1);
    request.onupgradeneeded = () =>
      request.result.createObjectStore('accounts', { keyPath: 'uid' });
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => {
      database = undefined;
      reject(new Error("Couldn't open local shopping storage"));
    };
  }));
}
export async function updateShoppingAccount(
  uid: string,
  change: (account: Account) => void,
): Promise<Account> {
  const database = await db();
  return new Promise((resolve, reject) => {
    const tx = database.transaction('accounts', 'readwrite');
    const store = tx.objectStore('accounts');
    const get = store.get(uid);
    let account: Account;
    get.onsuccess = () => {
      try {
        account = get.result || { uid, snapshot: EMPTY(), queue: [] };
        change(account);
        store.put(account);
      } catch (error) {
        tx.abort();
        reject(error);
      }
    };
    tx.oncomplete = () => resolve(account);
    tx.onerror = () =>
      reject(new Error("Couldn't save shopping on this device. Check free space."));
    tx.onabort = () => reject(new Error('The change was not saved on this device. Try again.'));
  });
}
export async function readShoppingMe(uid: string) {
  return (await updateShoppingAccount(uid, () => {})).me;
}
export async function saveShoppingMe(uid: string, me: Me) {
  await updateShoppingAccount(uid, (a) => {
    a.me = me;
  });
}

export function projectShopping(account: Account): ShoppingSnapshot {
  const snapshot = structuredClone(account.snapshot);
  for (const operation of account.queue) {
    if (operation.kind === 'upload') {
      if (!snapshot.receipts.some((r) => r.id === operation.id))
        snapshot.receipts.unshift({
          id: operation.id,
          merchant: 'Receipt on this device',
          date: operation.createdAt.slice(0, 10),
          currency: 'EUR',
          total: 0,
          items: [],
          status: 'queued',
          createdAt: operation.createdAt,
          issues: operation.blocked ? [operation.blocked] : [],
        });
      continue;
    }
    const action = operation.command.action;
    if (action.kind === 'add')
      for (const item of action.items) {
        if (!snapshot.list.some((i) => i.id === item.id)) {
          const product = snapshot.catalog.find(
            (p) => p.name.toLocaleLowerCase() === item.name.toLocaleLowerCase(),
          );
          snapshot.list.push({
            ...item,
            productId: product?.id,
            category: product?.category || 'other',
            done: false,
            author: 'You',
            revision: 1,
          });
        }
      }
    if (action.kind === 'mark')
      snapshot.list = snapshot.list.map((i) =>
        i.id === action.id ? { ...i, done: action.done, revision: (i.revision || 1) + 1 } : i,
      );
    if (action.kind === 'remove') snapshot.list = snapshot.list.filter((i) => i.id !== action.id);
    if (action.kind === 'product')
      snapshot.catalog = snapshot.catalog.map((p) =>
        p.id === action.product.id ? { ...action.product, revision: (p.revision || 1) + 1 } : p,
      );
    if (action.kind === 'import' && !snapshot.imports.some((i) => i.id === action.id))
      snapshot.imports.push({ id: action.id, text: action.text, status: 'local' });
  }
  return snapshot;
}
export type ShoppingAdapter = ReturnType<typeof useShopping>;
export function useShopping() {
  const uid = currentUserId();
  const [account, setAccount] = useState<Account>({ uid, snapshot: EMPTY(), queue: [] });
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState('');
  const [online, setOnline] = useState(navigator.onLine);
  const [syncing, setSyncing] = useState(false);
  const [photoCache, setPhotoCache] = useState<Record<string, string[]>>({});
  const [photoErrors, setPhotoErrors] = useState<Record<string, string>>({});
  const [queuedPhotos, setQueuedPhotos] = useState<Record<string, string[]>>({});
  const running = useRef(false);
  useEffect(() => {
    const photos = Object.fromEntries(
      account.queue
        .filter((p): p is Extract<PendingShopping, { kind: 'upload' }> => p.kind === 'upload')
        .map((p) => [p.id, p.photos.map((blob) => URL.createObjectURL(blob))]),
    );
    setQueuedPhotos(photos);
    return () =>
      Object.values(photos)
        .flat()
        .forEach((url) => URL.revokeObjectURL(url));
  }, [account.queue]);
  const active = useRef(true);
  const apply = useCallback(
    async (change: (account: Account) => void) => {
      const next = await updateShoppingAccount(uid, change);
      if (active.current) setAccount(next);
      return next;
    },
    [uid],
  );
  const refresh = useCallback(async () => {
    const snapshot = await apiAt<ShoppingSnapshot>('/juggle-shopping/v1/state');
    await apply((account) => {
      account.snapshot = snapshot;
    });
    if (active.current) {
      setOnline(true);
      setError('');
    }
    return snapshot;
  }, [apply]);
  const sync = useCallback(async () => {
    if (running.current || !navigator.onLine || !uid) return;
    running.current = true;
    if (active.current) setSyncing(true);
    try {
      // Replays are safe across tabs; operation UUIDs and photo digests are checked by the server.
      const flush = async () => {
        for (;;) {
          const current = await updateShoppingAccount(uid, () => {});
          const operation = current.queue[0];
          if (!operation || operation.blocked) break;
          try {
            if (operation.kind === 'command')
              await apiAt('/juggle-shopping/v1/commands', 'POST', operation.command);
            else {
              const body = new FormData();
              body.set('operation_id', operation.id);
              operation.photos.forEach((blob, index) =>
                body.append('photo', blob, `receipt-${index + 1}.jpg`),
              );
              await apiAt('/juggle-shopping/v1/receipts', 'POST', body);
            }
            // Install server state and remove this operation atomically. Lost responses replay the same ID.
            const snapshot = await apiAt<ShoppingSnapshot>('/juggle-shopping/v1/state');
            await apply((a) => {
              a.snapshot = snapshot;
              a.queue = a.queue.filter((p) => p.id !== operation.id);
            });
          } catch (error) {
            if (
              error instanceof ApiError &&
              error.status > 0 &&
              error.status < 500 &&
              error.status !== 429
            ) {
              await apply((a) => {
                const p = a.queue.find((p) => p.id === operation.id);
                if (p) p.blocked = error.message;
              });
            }
            throw error;
          }
        }
        await refresh();
      };
      if (navigator.locks)
        await navigator.locks.request(`crew-juggle-shopping:${uid}`, { mode: 'exclusive' }, flush);
      else await flush();
    } catch (error) {
      if (active.current) {
        setError(error instanceof Error ? error.message : "Couldn't sync shopping");
        if (error instanceof ApiError && error.status === 0) setOnline(false);
      }
    } finally {
      running.current = false;
      if (active.current) setSyncing(false);
    }
  }, [uid, apply, refresh]);
  useEffect(() => {
    active.current = true;
    void updateShoppingAccount(uid, () => {})
      .then((a) => {
        if (active.current) {
          setAccount(a);
          setLoaded(true);
        }
        return sync();
      })
      .catch((e) => {
        if (active.current) {
          setLoaded(true);
          setError(e.message);
        }
      });
    const interval = setInterval(() => void sync(), 5000);
    const connection = () => {
      setOnline(navigator.onLine);
      if (navigator.onLine) void sync();
    };
    const focus = () => void sync();
    window.addEventListener('online', connection);
    window.addEventListener('offline', connection);
    window.addEventListener('focus', focus);
    return () => {
      active.current = false;
      clearInterval(interval);
      window.removeEventListener('online', connection);
      window.removeEventListener('offline', connection);
      window.removeEventListener('focus', focus);
    };
  }, [uid, sync]);
  const enqueue = async (action: Action) => {
    const id = crypto.randomUUID();
    await apply((a) => {
      a.queue.push({ id, kind: 'command', command: { operation_id: id, action } });
    });
    void sync();
  };
  const data = projectShopping(account);
  data.receipts = data.receipts.map((r) => ({
    ...r,
    photos: queuedPhotos[r.id] || photoCache[r.id],
    photoError: photoErrors[r.id],
  }));
  async function addItems(items: ListItem[]) {
    await enqueue({
      kind: 'add',
      items: items.map((i) => ({ id: i.id, name: i.name, quantity: i.quantity })),
    });
  }
  function setList(update: SetStateAction<ListItem[]>) {
    const next = typeof update === 'function' ? update(data.list) : update;
    const changes: Action[] = [];
    for (const old of data.list) {
      const changed = next.find((i) => i.id === old.id);
      if (!changed) changes.push({ kind: 'remove', id: old.id, revision: old.revision || 1 });
      else if (old.done !== changed.done)
        changes.push({ kind: 'mark', id: old.id, revision: old.revision || 1, done: changed.done });
    }
    void apply((a) => {
      for (const action of changes) {
        if (action.kind === 'mark' || action.kind === 'remove') {
          const current = projectShopping(a).list.find((i) => i.id === action.id);
          if (!current || (action.kind === 'mark' && current.done === action.done)) continue;
          action.revision = current.revision || 1;
        }
        const id = crypto.randomUUID();
        a.queue.push({ id, kind: 'command', command: { operation_id: id, action } });
      }
    })
      .then(() => sync())
      .catch((e) => setError(e.message));
  }
  function setCatalog(update: SetStateAction<Product[]>) {
    const next = typeof update === 'function' ? update(data.catalog) : update;
    const changed = next.find((p) => {
      const old = data.catalog.find((i) => i.id === p.id);
      return (
        old &&
        (old.category !== p.category || old.subcategory !== p.subcategory || old.name !== p.name)
      );
    });
    if (changed)
      void enqueue({ kind: 'product', product: changed }).catch((e) => setError(e.message));
  }
  async function uploadPhotos(urls: string[]) {
    if (!urls.length || urls.length > 5) throw new Error('Pick 1 to 5 photos of the receipt');
    const photos = await Promise.all(
      urls.map(async (url) => {
        if (!url.startsWith('blob:')) throw new Error('Pick a photo from this device');
        return (await fetch(url)).blob();
      }),
    );
    const id = crypto.randomUUID();
    await apply((a) => {
      a.queue.push({ id, kind: 'upload', photos, createdAt: new Date().toISOString() });
    });
    void sync();
  }
  async function editReceipt(receipt: Receipt, confirmDuplicate = false, allowIncomplete = false) {
    if (!online) throw new Error('Go online to fix the receipt');
    const { photos: _photos, photoError: _photoError, ...metadata } = receipt;
    await apiAt('/juggle-shopping/v1/commands', 'POST', {
      operation_id: crypto.randomUUID(),
      action: {
        kind: 'edit_receipt',
        receipt: metadata,
        confirm_duplicate: confirmDuplicate,
        allow_incomplete: allowIncomplete,
      },
    });
    const snapshot = await refresh();
    return snapshot.receipts.find((r) => r.id === receipt.id);
  }
  async function retryReceipt(receipt: Receipt) {
    await apiAt('/juggle-shopping/v1/commands', 'POST', {
      operation_id: crypto.randomUUID(),
      action: { kind: 'retry', id: receipt.id, revision: receipt.revision },
    });
    await refresh();
  }
  async function deleteReceipt(receipt: Receipt) {
    await apiAt('/juggle-shopping/v1/commands', 'POST', {
      operation_id: crypto.randomUUID(),
      action: { kind: 'delete_receipt', id: receipt.id, revision: receipt.revision },
    });
    await refresh();
  }
  async function loadPhotos(id: string) {
    if (photoCache[id]?.length) return;
    if (account.queue.some((p) => p.id === id && p.kind === 'upload')) return;
    try {
      const result = await apiAt<{ photos: string[]; expired: boolean }>(
        `/juggle-shopping/v1/receipts/${encodeURIComponent(id)}/photos`,
      );
      if (active.current) {
        setPhotoCache((previous) => ({ ...previous, [id]: result.photos }));
        setPhotoErrors((previous) => ({
          ...previous,
          [id]: result.photos.length ? '' : 'The photo is gone. It may have expired.',
        }));
      }
    } catch (e) {
      if (active.current)
        setPhotoErrors((previous) => ({
          ...previous,
          [id]: e instanceof Error ? e.message : "Couldn't open the photo",
        }));
    }
  }
  async function discardPending(id: string) {
    await apply((a) => {
      a.queue = a.queue.filter((p) => p.id !== id);
    });
    void sync();
  }
  async function retryPending(id: string) {
    await refresh();
    await apply((a) => {
      const pending = a.queue.find((p) => p.id === id);
      if (!pending) return;
      pending.blocked = undefined;
      if (pending.kind === 'command') {
        const action = pending.command.action;
        if (action.kind === 'mark' || action.kind === 'remove') {
          const current = a.snapshot.list.find((i) => i.id === action.id);
          if (current) action.revision = current.revision || 1;
        }
        if (action.kind === 'product') {
          const current = a.snapshot.catalog.find((p) => p.id === action.product.id);
          if (current) action.product.revision = current.revision;
        }
        pending.command.operation_id = crypto.randomUUID();
      }
    });
    void sync();
  }
  return {
    data,
    loaded,
    error,
    online,
    syncing,
    pending: account.queue,
    setList,
    setCatalog,
    addItems,
    uploadPhotos,
    editReceipt,
    saveRule: async (rule: CorrectionRule) => {
      if (!online) throw new Error('Go online to change the rule');
      await apiAt('/juggle-shopping/v1/commands', 'POST', {
        operation_id: crypto.randomUUID(),
        action: { kind: 'rule', rule },
      });
      await refresh();
    },
    deleteRule: async (rule: CorrectionRule) => {
      if (!online) throw new Error('Go online to delete the rule');
      await apiAt('/juggle-shopping/v1/commands', 'POST', {
        operation_id: crypto.randomUUID(),
        action: { kind: 'rule_delete', id: rule.id, revision: rule.revision },
      });
      await refresh();
    },
    retryReceipt,
    deleteReceipt,
    loadPhotos,
    sync,
    discardPending,
    retryPending,
    importText: (text: string) => enqueue({ kind: 'import', id: crypto.randomUUID(), text }),
  };
}
