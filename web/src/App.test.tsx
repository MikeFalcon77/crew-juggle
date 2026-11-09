import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import App from './App';
import { api, apiAt } from './api';
import * as session from './session';
import type { Day, Me } from './types';
import { crewMe, flatDay } from './testFixtures';

vi.mock('./session', () => ({
  loadConfig: vi.fn(),
  watch: vi.fn(),
  signInAsDemo: vi.fn(async () => undefined),
  signInWithGoogle: vi.fn(),
  signOut: vi.fn(),
  googleConfigured: () => false,
  signInMode: () => 'demo',
  token: async () => 'demo.pip.x',
}));
vi.mock('./api', async (original) => ({
  ...(await original<typeof import('./api')>()),
  api: vi.fn(),
  apiAt: vi.fn(),
}));

let me: Me;
let day: Day;
let signedIn = true;

function mount(path = '/routines?tab=today') {
  history.replaceState({}, '', path);
  render(
    <QueryClientProvider
      client={
        new QueryClient({
          defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
        })
      }
    >
      <App />
    </QueryClientProvider>,
  );
}

afterEach(cleanup);
beforeEach(() => {
  me = structuredClone(crewMe);
  day = flatDay();
  signedIn = true;
  try {
    localStorage.clear();
  } catch {
    /* ignore */
  }
  vi.mocked(session.loadConfig).mockResolvedValue({
    sign_in: 'demo',
    demo_users: [
      { key: 'pip', name: 'Pip', crew: 'Flat 4B', tagline: 'Plans everything.' },
      { key: 'zed', name: 'Zed', crew: 'Flat 4B', tagline: 'Builds robots.' },
    ],
  });
  vi.mocked(session.watch).mockImplementation((callback) => {
    callback(signedIn, signedIn ? 'demo-pip' : undefined);
    return () => {};
  });
  vi.mocked(apiAt).mockImplementation(async (path) => {
    if (path === '/juggle-crews/v1/me') return me as never;
    if (path === '/juggle-shopping/v1/state') return { list: [], receipts: [] } as never;
    throw new Error(`unexpected ${path}`);
  });
  vi.mocked(api).mockReset();
  vi.mocked(api).mockImplementation(async (path, method = 'GET', body) => {
    if (path === '/me')
      return {
        uid: 'demo-pip',
        timezone: 'Europe/Berlin',
        today: '2026-11-10',
        vapid_public_key: '',
      } as never;
    if (path === '/me/preferences')
      return { preferences: { timezone: 'Europe/Berlin', revision: 1 }, members: [] } as never;
    if (path === '/day') return day as never;
    if (path === '/people') return { people: day.people, today: day.today } as never;
    if (method === 'PUT' && path.startsWith('/occurrences/')) {
      const id = path.split('/').at(-1)!;
      const item = day.items.find((i) => i.routine.id === id)!;
      const request = body as { completed: boolean };
      const saved = {
        ...item,
        revision: item.revision + 1,
        status: request.completed ? 'done' : 'pending',
        completed_by: request.completed ? 'demo-pip' : null,
        completed_at: request.completed ? '2026-11-10T09:31:00Z' : null,
      };
      day = { ...day, items: day.items.map((i) => (i.id === item.id ? (saved as typeof i) : i)) };
      return saved as never;
    }
    throw new Error(`unexpected ${method} ${path}`);
  });
});

describe('the app', () => {
  it('offers the demo characters when nobody is signed in', async () => {
    signedIn = false;
    mount('/today');
    expect(await screen.findByText('Demo · pick a character')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: /Zed/ }));
    expect(session.signInAsDemo).toHaveBeenCalledWith('zed');
  });

  it('asks a signed-in account without a crew to start or join one', async () => {
    me = { ...me, crew: null };
    mount('/join?code=K7QX-M2PA');
    expect(await screen.findByRole('heading', { name: 'Join your crew' })).toBeTruthy();
    expect((screen.getByLabelText('Invite code') as HTMLInputElement).value).toBe('K7QX-M2PA');
  });

  it('asks before undoing a crewmate’s check-off and names them', async () => {
    mount();
    const row = await screen.findByRole('group', {
      name: 'Done: Supervise kitchen cleaning 😼, Mochi, 21:00',
    });
    fireEvent.click(within(row).getByRole('button', { name: /^Undo/ }));
    const dialog = screen.getByRole('dialog');
    expect(within(dialog).getByText("Undo Zed's check-off?")).toBeTruthy();
    expect(api).not.toHaveBeenCalledWith(
      expect.stringMatching(/^\/occurrences/),
      'PUT',
      expect.anything(),
    );
    fireEvent.click(within(dialog).getByRole('button', { name: 'Undo' }));
    await waitFor(() =>
      expect(api).toHaveBeenCalledWith(
        '/occurrences/2026-11-10/supervise',
        'PUT',
        expect.objectContaining({ completed: false, confirmed_completed_by: 'demo-zed' }),
      ),
    );
  });

  it('celebrates only after the check-off is saved', async () => {
    mount();
    fireEvent.click(
      await screen.findByRole('button', { name: 'Check off: Take out the trash, Pip, 19:30' }),
    );
    expect(await screen.findByText('Day cleared. Legend.')).toBeTruthy();
  });
});
