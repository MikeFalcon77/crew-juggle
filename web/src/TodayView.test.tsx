import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, within } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import TodayView from './TodayView';
import { apiAt } from './api';
import { setCrewZone } from './time';
import { flatDay } from './testFixtures';

vi.mock('./api', async (original) => ({
  ...(await original<typeof import('./api')>()),
  api: vi.fn(),
  apiAt: vi.fn(),
}));

afterEach(cleanup);
beforeEach(() => {
  setCrewZone('Europe/Berlin');
  vi.mocked(apiAt).mockResolvedValue({
    list: [{ name: 'Cat food', done: false, author: 'Pip' }],
    receipts: [],
  } as never);
});

function mount() {
  const onMark = vi.fn();
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <TodayView
        day={flatDay()}
        crewName="Flat 4B"
        routinesLoading={false}
        routinesError={false}
        online
        saving={null}
        onMark={onMark}
        onNavigate={vi.fn()}
        onPerson={vi.fn()}
      />
    </QueryClientProvider>,
  );
  return onMark;
}

describe('Today', () => {
  it('puts Zed’s overdue dishes first and lets you check them off from the card', async () => {
    const onMark = mount();
    const attention = screen.getByRole('region', { name: 'Needs attention' });
    expect(within(attention).getByText('1 overdue')).toBeTruthy();
    fireEvent.click(
      within(attention).getByRole('button', { name: 'Check off Do the dishes for Zed' }),
    );
    expect(onMark).toHaveBeenCalledWith(expect.objectContaining({ id: '2026-11-10/dishes' }));
  });

  it('shows the shopping list next to the routines', async () => {
    mount();
    expect(await screen.findByText('1 thing to buy')).toBeTruthy();
    expect(screen.getByText('Cat food')).toBeTruthy();
    expect(screen.getByRole('heading', { name: 'Today' })).toBeTruthy();
    expect(screen.getByText(/FLAT 4B · TODAY/)).toBeTruthy();
  });
});
