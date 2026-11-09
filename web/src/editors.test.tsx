import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import RoutineEditor, { routineId } from './RoutineEditor';
import { api } from './api';
import { flatSchedule, people } from './testFixtures';

vi.mock('./api', async (original) => ({
  ...(await original<typeof import('./api')>()),
  api: vi.fn(),
}));

afterEach(cleanup);
beforeEach(() => {
  vi.mocked(api).mockReset();
  vi.mocked(api).mockResolvedValue({} as never);
});

function mount(version = null as ReturnType<typeof flatSchedule>['versions'][number] | null) {
  const onClose = vi.fn();
  render(
    <QueryClientProvider client={new QueryClient()}>
      <RoutineEditor
        version={version}
        versions={flatSchedule().versions}
        people={people}
        initialPersonId="p-zed"
        today="2026-11-10"
        disabled={false}
        onClose={onClose}
        onError={vi.fn()}
      />
    </QueryClientProvider>,
  );
  return onClose;
}

describe('the routine editor', () => {
  it('saves the picked weekdays and gives the routine a readable id', async () => {
    const onClose = mount();
    fireEvent.change(screen.getByLabelText('What'), { target: { value: 'Water the cactus' } });
    fireEvent.click(screen.getByRole('button', { name: 'Saturday' }));
    fireEvent.click(screen.getByRole('button', { name: 'Wednesday' }));
    expect(screen.getByText('Wed, Sat')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: /Save/ }));
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    const body = vi.mocked(api).mock.calls[0][2] as {
      routine: { id: string; days: number[]; person_id: string };
    };
    expect(body.routine.days).toEqual([3, 6]);
    expect(body.routine.person_id).toBe('p-zed');
    expect(body.routine.id).toMatch(/^water-the-cactus-[0-9a-f]{6}$/);
  });

  it('turns a new routine into a second step that borrows who and when', async () => {
    mount();
    fireEvent.change(screen.getByLabelText('What'), { target: { value: 'Feed Mochi a treat' } });
    fireEvent.change(screen.getByLabelText('Second step of'), { target: { value: 'dishes' } });
    expect(screen.queryByLabelText('Who')).toBeNull();
    expect(screen.queryByLabelText('Time')).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: /Save/ }));
    await waitFor(() => expect(api).toHaveBeenCalled());
    const body = vi.mocked(api).mock.calls[0][2] as { routine: { prerequisite: string } };
    expect(body.routine.prerequisite).toBe('dishes');
  });

  it('keeps the revision of the version that was opened', async () => {
    const dishes = flatSchedule().versions.find((v) => v.routine.id === 'dishes')!;
    mount({ ...dishes, routine: { ...dishes.routine, revision: 4 } });
    fireEvent.click(screen.getByRole('button', { name: /Save/ }));
    await waitFor(() => expect(api).toHaveBeenCalled());
    const body = vi.mocked(api).mock.calls[0][2] as {
      expected_revision: number;
      effective_from: string;
    };
    expect(body.expected_revision).toBe(4);
    expect(body.effective_from).toBe('2026-11-11');
  });

  it('builds safe ids from any name', () => {
    expect(routineId('  Ünïcode & Emoji 😼 ')).toMatch(/^unicode-emoji-[0-9a-f]{6}$/);
    expect(routineId('😼')).toMatch(/^routine-[0-9a-f]{6}$/);
  });
});
