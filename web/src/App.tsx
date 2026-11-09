import { useEffect, useRef, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  Bell,
  CalendarDays,
  ListChecks,
  LogOut,
  RefreshCw,
  Settings,
  ShieldCheck,
  ShoppingBag,
  UsersRound,
  WifiOff,
  X,
  Globe2,
} from 'lucide-react';
import { api, apiAt, ApiError } from './api';
import * as session from './session';
import CrewView from './CrewView';
import LoginView from './LoginView';
import Onboarding from './Onboarding';
import RoutinesPage from './RoutinesPage';
import SettingsView from './SettingsView';
import ShoppingView from './ShoppingView';
import { readShoppingMe, saveShoppingMe } from './shoppingStore';
import TodayView from './TodayView';
import { useAppRoute, type AppRoute } from './routes';
import UndoDialog, { sameCompletion } from './UndoDialog';
import { browserZone, setCrewZone, zoneName } from './time';
import { disablePush, registerPush, ring } from './push';
import { celebrationFor, type Celebration } from './celebration';
import type { CrewPreferences, Day, Me, Occurrence, RoutinesMe } from './types';

const PUSH_ID = 'crew-juggle-push-id';
const SOUND = 'crew-juggle-sound';
function stored(key: string) {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}
function store(key: string, value: string | null) {
  try {
    if (value === null) localStorage.removeItem(key);
    else localStorage.setItem(key, value);
  } catch {
    /* Private mode: settings simply do not persist. */
  }
}

export default function App() {
  const client = useQueryClient();
  const config = useQuery({
    queryKey: ['session-config'],
    queryFn: session.loadConfig,
    staleTime: Infinity,
  });
  const [authUid, setAuthUid] = useState('');
  const authUidRef = useRef('');
  const [cachedMe, setCachedMe] = useState<Me>();
  const [signedIn, setSignedIn] = useState<boolean | null>(null);
  const [signingIn, setSigningIn] = useState(false);
  const { route, navigate: changeRoute } = useAppRoute();
  const page = route.page;
  const [online, setOnline] = useState(navigator.onLine);
  const [error, setError] = useState('');
  const [pushId, setPushId] = useState(stored(PUSH_ID));
  const [sound, setSound] = useState(stored(SOUND) === 'true');
  const [pushBusy, setPushBusy] = useState(false);
  const [saving, setSaving] = useState<string | null>(null);
  const [celebration, setCelebration] = useState<Celebration | null>(null);
  const [confirmation, setConfirmation] = useState<Occurrence | null>(null);

  useEffect(() => {
    if (!celebration) return;
    const timer = setTimeout(
      () => setCelebration(null),
      celebration.level === 'item' ? 1100 : 1900,
    );
    return () => clearTimeout(timer);
  }, [celebration]);

  useEffect(() => {
    if (!config.data) return;
    return session.watch((value, uid) => {
      if (uid && uid !== authUidRef.current) {
        client.removeQueries({ predicate: (q) => q.queryKey[0] !== 'session-config' });
        authUidRef.current = uid;
        setCachedMe(undefined);
        setAuthUid(uid);
        void readShoppingMe(uid)
          .then((profile) => {
            if (authUidRef.current === uid) setCachedMe(profile as Me | undefined);
          })
          .catch(() => {});
      }
      if (!value) {
        authUidRef.current = '';
        setAuthUid('');
        setCachedMe(undefined);
        client.removeQueries({ predicate: (q) => q.queryKey[0] !== 'session-config' });
      }
      setSignedIn(value);
    });
  }, [client, config.data]);

  const liveMe = useQuery({
    queryKey: ['me'],
    queryFn: () => apiAt<Me>('/juggle-crews/v1/me'),
    enabled: signedIn === true,
    refetchInterval: 15000,
  });
  const offline =
    !online || (liveMe.isError && liveMe.error instanceof ApiError && liveMe.error.status === 0);
  const me =
    liveMe.data ||
    (page === 'shopping' && offline && cachedMe?.uid === authUid ? cachedMe : undefined);
  const crew = me?.crew || null;
  useEffect(() => {
    if (crew) setCrewZone(crew.timezone);
  }, [crew]);
  useEffect(() => {
    if (liveMe.data && authUid) void saveShoppingMe(authUid, liveMe.data).catch(() => {});
  }, [liveMe.data, authUid]);

  const routinesMe = useQuery({
    queryKey: ['routines-me'],
    queryFn: () => api<RoutinesMe>('/me'),
    enabled: Boolean(crew),
  });
  const preferences = useQuery({
    queryKey: ['preferences'],
    queryFn: () => api<CrewPreferences>('/me/preferences'),
    enabled: Boolean(crew),
    refetchInterval: 15000,
  });
  const isHistory = page === 'routines' && route.tab === 'history' && Boolean(route.date);
  const day = useQuery({
    queryKey: ['day', isHistory ? route.date : 'today'],
    queryFn: () => api<Day>(isHistory ? `/day?date=${route.date}` : '/day'),
    enabled: Boolean(crew) && (page === 'today' || page === 'routines'),
    refetchInterval: 5000,
  });

  useEffect(() => {
    const update = () => {
      setOnline(navigator.onLine);
      if (navigator.onLine) void client.invalidateQueries();
    };
    window.addEventListener('online', update);
    window.addEventListener('offline', update);
    return () => {
      window.removeEventListener('online', update);
      window.removeEventListener('offline', update);
    };
  }, [client]);
  useEffect(() => {
    const handler = (event: MessageEvent) => {
      if (event.data?.type === 'routines-reminder') {
        void client.invalidateQueries({ queryKey: ['day'] });
        if (sound) {
          try {
            ring();
          } catch {
            /* Sound needs a gesture in this session. */
          }
        }
      }
      if (event.data?.type === 'shopping-update') {
        void client.invalidateQueries({ queryKey: ['shopping-glance'] });
      }
    };
    navigator.serviceWorker?.addEventListener('message', handler);
    return () => navigator.serviceWorker?.removeEventListener('message', handler);
  }, [client, sound]);

  const mark = useMutation({
    mutationFn: ({
      item,
      operationId,
      confirmedBy,
    }: {
      item: Occurrence;
      operationId: string;
      confirmedBy?: string;
    }) =>
      api<Occurrence>(`/occurrences/${item.id.split('/')[0]}/${item.routine.id}`, 'PUT', {
        operation_id: operationId,
        expected_revision: item.revision,
        completed: item.status !== 'done',
        confirmed_completed_by: confirmedBy,
      }),
    onSuccess: async (result, variables) => {
      if (
        day.data &&
        day.data.date === result.id.split('/')[0] &&
        variables.item.status === 'pending' &&
        result.status === 'done'
      ) {
        const before = {
          ...day.data,
          items: day.data.items.map((i) => (i.id === variables.item.id ? variables.item : i)),
        };
        setCelebration(celebrationFor(before, result, variables.operationId));
      }
      setError('');
      setConfirmation(null);
      await client.invalidateQueries({ queryKey: ['day'] });
    },
    onError: (err) => {
      setConfirmation(null);
      setError(err.message);
      void client.invalidateQueries({ queryKey: ['day'] });
    },
    onSettled: () => setSaving(null),
  });
  useEffect(() => {
    if (
      confirmation &&
      day.data &&
      !sameCompletion(
        confirmation,
        day.data.items.find((i) => i.id === confirmation.id),
      )
    ) {
      setConfirmation(null);
      setError('Someone changed that on another device. Check the list and try again.');
    }
  }, [confirmation, day.data]);

  function act(item: Occurrence, confirmedBy?: string) {
    if (saving || !navigator.onLine || day.isError || !me) return;
    if (item.status === 'done' && item.completed_by !== me.uid && !confirmedBy) {
      setConfirmation(structuredClone(item));
      return;
    }
    setSaving(item.id);
    mark.mutate({ item, operationId: crypto.randomUUID(), confirmedBy });
  }
  function confirmUndo() {
    if (!confirmation) return;
    if (
      !sameCompletion(
        confirmation,
        day.data?.items.find((i) => i.id === confirmation.id),
      )
    ) {
      setConfirmation(null);
      setError('That changed in the meantime. Confirm again.');
      void client.invalidateQueries({ queryKey: ['day'] });
      return;
    }
    act(confirmation, confirmation.completed_by || undefined);
  }
  async function signOut() {
    try {
      if (pushId) await disablePush();
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setPushId(null);
      await session.signOut();
      client.removeQueries({ predicate: (q) => q.queryKey[0] !== 'session-config' });
    }
  }
  async function pushAction(action: string) {
    setPushBusy(true);
    setError('');
    try {
      if (action === 'enable') setPushId(await registerPush(routinesMe.data!.vapid_public_key));
      if (action === 'disable') {
        await disablePush();
        setPushId(null);
      }
      if (action === 'test') {
        await api('/push/test', 'POST', { id: pushId });
        if (sound) ring();
      }
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setPushBusy(false);
    }
  }
  function navigate(next: AppRoute) {
    setConfirmation(null);
    setCelebration(null);
    setError('');
    changeRoute(next);
  }

  if (config.isPending || signedIn === null)
    return (
      <div className="initial-loading">
        <span className="brand-symbol">
          <img src="/icon.svg" alt="" width="45" height="45" />
        </span>
        <span className="spinner" />
      </div>
    );
  if (!signedIn || !me)
    return (
      <LoginView
        mode={config.data!.sign_in}
        demoUsers={config.data!.demo_users}
        googleReady={session.googleConfigured()}
        busy={signingIn || (signedIn === true && liveMe.isFetching)}
        error={
          error ||
          (signedIn && liveMe.isError
            ? liveMe.error instanceof ApiError && liveMe.error.status === 401
              ? 'This account cannot sign in here.'
              : liveMe.error.message
            : '')
        }
        onGoogle={() => {
          setError('');
          setSigningIn(true);
          void session
            .signInWithGoogle()
            .catch((e) => setError(e.message))
            .finally(() => setSigningIn(false));
        }}
        onDemo={(key) => {
          setError('');
          setSigningIn(true);
          void session
            .signInAsDemo(key)
            .catch((e) => setError(e.message))
            .finally(() => setSigningIn(false));
        }}
      />
    );
  if (!crew)
    return (
      <Onboarding
        me={me}
        initialCode={page === 'join' ? route.code : undefined}
        onSignOut={() => void signOut()}
      />
    );

  const zone = preferences.data?.preferences.timezone || crew.timezone;
  const disabled = !online || ((page === 'today' || page === 'routines') && day.isError);
  const nav: [AppRoute['page'], string, typeof ListChecks][] = [
    ['today', 'Today', CalendarDays],
    ['routines', 'Routines', ListChecks],
    ['shopping', 'Shopping', ShoppingBag],
    ['crew', 'Crew', UsersRound],
  ];
  return (
    <div className="app">
      <header className="topbar">
        <a
          className="brand"
          href="/today"
          onClick={(e) => {
            e.preventDefault();
            navigate({ page: 'today' });
          }}
        >
          <span className="brand-symbol">
            <img src="/icon.svg" alt="" width="45" height="45" />
          </span>
          <span>
            crew-juggle<small>{crew.name.toUpperCase()}</small>
          </span>
        </a>
        <nav className="desktop-nav" aria-label="Sections">
          {nav.map(([id, label]) => (
            <button
              className={page === id ? 'active' : ''}
              onClick={() => navigate({ page: id })}
              key={id}
            >
              {label}
            </button>
          ))}
        </nav>
        <div className="header-actions">
          <button
            className="icon-button"
            aria-label="Settings"
            onClick={() => navigate({ page: 'settings', section: 'about' })}
          >
            <Settings size={20} />
          </button>
          <button
            className={`icon-button notifications-toggle ${pushId ? 'enabled' : ''}`}
            onClick={() => navigate({ page: 'settings', section: 'notifications' })}
            aria-label="Notifications"
          >
            <Bell size={20} />
            {pushId && <span className="notification-dot" />}
          </button>
          <span className="member-label">
            {crew.members.find((m) => m.uid === me.uid)?.display_name || me.name}
          </span>
          <button className="icon-button" onClick={() => void signOut()} aria-label="Sign out">
            <LogOut size={18} />
          </button>
        </div>
      </header>
      <main>
        {disabled && (
          <div className="notice offline" role="status">
            <WifiOff size={18} />
            <span>
              {online ? "Can't reach the server. The list may be out of date." : "You're offline."}{' '}
              Check-offs are paused.
            </span>
            <button
              className="icon-button"
              onClick={() => void client.invalidateQueries()}
              aria-label="Retry"
            >
              <RefreshCw size={16} />
            </button>
          </div>
        )}
        {error && (
          <div className="notice error" role="alert">
            {error}
            <button className="icon-button" onClick={() => setError('')} aria-label="Dismiss">
              <X size={16} />
            </button>
          </div>
        )}
        {preferences.data &&
          page !== 'settings' &&
          preferences.data.preferences.revision === 0 &&
          browserZone() !== crew.timezone && (
            <div className="notice timezone-notice">
              <Globe2 size={18} />
              <span>
                This device is on {zoneName(browserZone())} time; the crew runs on{' '}
                {zoneName(crew.timezone)}.
              </span>
              <button
                className="text-button"
                onClick={() => navigate({ page: 'settings', section: 'time' })}
              >
                Show my local times
              </button>
            </div>
          )}
        {page === 'settings' ? (
          <SettingsView
            data={preferences.data}
            preferencesError={preferences.isError}
            myUid={me.uid}
            disabled={!online}
            onError={setError}
            section={route.section}
            onNavigate={navigate}
            deviceId={pushId}
            notifications={{
              pushId,
              busy: pushBusy,
              online,
              sound,
              onPush: (action) => void pushAction(action),
              onSound: () => {
                const next = !sound;
                setSound(next);
                store(SOUND, String(next));
                if (next) ring();
              },
            }}
          />
        ) : page === 'today' ? (
          <TodayView
            day={day.data}
            crewName={crew.name}
            celebration={celebration}
            online={online}
            routinesLoading={day.isPending}
            routinesError={day.isError}
            saving={saving}
            onMark={act}
            onNavigate={navigate}
            zone={zone}
            person={route.person || ''}
            onPerson={(person) => navigate({ page: 'today', person: person || undefined })}
          />
        ) : page === 'shopping' ? (
          <ShoppingView key={authUid} route={route} onNavigate={navigate} />
        ) : page === 'crew' ? (
          <CrewView me={me} disabled={!online} onError={setError} />
        ) : (
          <RoutinesPage
            route={route}
            day={day.data}
            dayPending={day.isPending}
            myUid={me.uid}
            zone={zone}
            disabled={disabled}
            saving={saving}
            celebration={celebration}
            onMark={(item) => act(item)}
            onNavigate={navigate}
            onError={setError}
          />
        )}
        <footer className="app-footer">
          <span>
            <ShieldCheck size={13} />
            Only {crew.name} sees this · {zoneName(crew.timezone)} time
          </span>
          <span>
            {!online
              ? 'Offline'
              : day.isError
                ? 'Server unavailable'
                : 'Everyone sees changes live'}
          </span>
        </footer>
        {confirmation && day.data && (
          <UndoDialog
            item={confirmation}
            people={day.data.people}
            zone={zone}
            busy={Boolean(saving)}
            disabled={disabled}
            onClose={() => setConfirmation(null)}
            onConfirm={confirmUndo}
          />
        )}
      </main>
      <nav className="mobile-nav" aria-label="App sections">
        {nav.map(([id, label, Icon]) => (
          <button
            key={id}
            className={page === id ? 'active' : ''}
            onClick={() => navigate({ page: id })}
          >
            <Icon size={20} />
            <span>{label}</span>
          </button>
        ))}
      </nav>
    </div>
  );
}
