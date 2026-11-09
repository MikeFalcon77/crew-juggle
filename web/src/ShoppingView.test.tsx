import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import ShoppingView from './ShoppingView';
import { seedReceipts, CATALOG } from './shoppingFixtures';
import type { ShoppingSnapshot } from './shoppingStore';

const mocked = vi.hoisted(() => ({ data: null as unknown as ShoppingSnapshot }));
vi.mock('./shoppingStore', () => ({
  useShopping: () => ({
    data: mocked.data,
    loaded: true,
    online: true,
    pending: [],
    syncing: false,
    error: '',
    setList: vi.fn(),
    setCatalog: vi.fn(),
    loadPhotos: vi.fn(),
  }),
}));
afterEach(cleanup);
function snapshot(today: string): ShoppingSnapshot {
  const template = seedReceipts()[0];
  const purchase = (id: string, date: string, total: number) => ({
    ...template,
    id,
    merchant: id,
    date,
    total,
    items: [{ ...template.items[0], amount: total }],
    status: 'saved' as const,
  });
  const purchases = [
    purchase('November Store', '2026-11-01', 2894),
    purchase('Archive Store', '2025-11-01', 1000),
  ];
  return {
    revision: 1,
    today,
    list: [],
    catalog: CATALOG,
    receipts: purchases,
    purchases,
    imports: [],
    aiConfigured: true,
  };
}
it.each(['history', 'analytics'])(
  'follows the current month after cached October loads November in %s',
  (tab) => {
    mocked.data = snapshot('2026-10-31');
    const view = render(<ShoppingView route={{ page: 'shopping', tab }} onNavigate={() => {}} />);
    const name = tab === 'history' ? 'History period' : 'Period';
    expect(screen.getByRole('combobox', { name })).toHaveProperty('value', '2026-10');
    mocked.data = snapshot('2026-11-01');
    view.rerender(<ShoppingView route={{ page: 'shopping', tab }} onNavigate={() => {}} />);
    expect(screen.getByRole('combobox', { name })).toHaveProperty('value', '2026-11');
    if (tab === 'history')
      expect(screen.getByRole('button', { name: /November Store/ })).toBeTruthy();
    else expect(screen.getAllByText(/28\.94/).length).toBeGreaterThan(0);
  },
);
it('offers all receipt months and preserves an explicitly selected historical year across refresh', () => {
  mocked.data = snapshot('2026-11-01');
  const view = render(
    <ShoppingView route={{ page: 'shopping', tab: 'history' }} onNavigate={() => {}} />,
  );
  fireEvent.change(screen.getByRole('combobox', { name: 'History period' }), {
    target: { value: '2025-11' },
  });
  expect(screen.getByRole('heading', { name: 'November 1, 2025' })).toBeTruthy();
  expect(screen.getByRole('button', { name: /Archive Store/ })).toBeTruthy();
  mocked.data = snapshot('2026-11-02');
  view.rerender(
    <ShoppingView route={{ page: 'shopping', tab: 'history' }} onNavigate={() => {}} />,
  );
  expect(screen.getByRole('combobox', { name: 'History period' })).toHaveProperty(
    'value',
    '2025-11',
  );
  view.rerender(
    <ShoppingView route={{ page: 'shopping', tab: 'analytics' }} onNavigate={() => {}} />,
  );
  fireEvent.change(screen.getByRole('combobox', { name: 'Period' }), {
    target: { value: '2025-11' },
  });
  expect(screen.getAllByText(/10\.00/).length).toBeGreaterThan(0);
  expect(document.querySelector('.sp-analytics-stats')?.textContent).not.toContain('28.94');
});

it('keeps a selected period visible when a corrected receipt moves to another year', () => {
  mocked.data = snapshot('2026-11-01');
  const view = render(
    <ShoppingView route={{ page: 'shopping', tab: 'history' }} onNavigate={() => {}} />,
  );
  fireEvent.change(screen.getByRole('combobox', { name: 'History period' }), {
    target: { value: '2025-11' },
  });
  mocked.data = snapshot('2026-11-01');
  mocked.data.receipts = mocked.data.receipts.filter((r) => !r.date.startsWith('2025'));
  mocked.data.purchases = mocked.data.purchases.filter((r) => !r.date.startsWith('2025'));
  view.rerender(
    <ShoppingView route={{ page: 'shopping', tab: 'history' }} onNavigate={() => {}} />,
  );
  expect(screen.getByRole('combobox', { name: 'History period' })).toHaveProperty(
    'value',
    '2025-11',
  );
  expect(screen.getByRole('option', { name: 'November 2025' })).toBeTruthy();
  expect(screen.getByRole('heading', { name: 'Nothing like that' })).toBeTruthy();
});
