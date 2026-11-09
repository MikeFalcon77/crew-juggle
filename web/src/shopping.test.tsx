import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { ReceiptDialog } from './ShoppingComponents';
import { CATALOG, seedReceipts } from './shoppingFixtures';
import { projectShopping } from './shoppingStore';
import { setCrewZone } from './time';
import {
  summarize,
  parseMoney,
  money,
  currencyScale,
  crewDate,
  shoppingMonths,
} from './shoppingModels';
afterEach(cleanup);
it('offers today for a different OCR year without silently changing or saving the receipt', async () => {
  const original = {
    ...seedReceipts()[0],
    date: '2025-11-01',
    status: 'review' as const,
    revision: 1,
  };
  const onSave = vi.fn();
  render(
    <ReceiptDialog
      receipt={original}
      catalog={CATALOG}
      today="2026-11-01"
      demo={false}
      onSave={onSave}
      onClose={() => {}}
      onRepeat={() => {}}
    />,
  );
  expect(screen.getByRole('alert').textContent).toContain('2025');
  expect(screen.getByRole('alert').textContent).toContain('2026');
  expect(onSave).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole('button', { name: "Use today's date" }));
  expect(screen.getByLabelText('Date', { exact: true })).toHaveProperty('value', '2026-11-01');
  expect(onSave).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole('button', { name: 'Save changes' }));
  await waitFor(() => expect(onSave).toHaveBeenCalledOnce());
  expect(onSave.mock.calls[0][0].date).toBe('2026-11-01');
  expect(onSave.mock.calls[0][0].total).toBe(original.total);
});
it('lets the user retain the printed historical date explicitly', async () => {
  const original = {
    ...seedReceipts()[0],
    date: '2025-11-01',
    status: 'review' as const,
    revision: 1,
  };
  const onSave = vi.fn();
  render(
    <ReceiptDialog
      receipt={original}
      catalog={CATALOG}
      today="2026-11-01"
      demo={false}
      onSave={onSave}
      onClose={() => {}}
      onRepeat={() => {}}
    />,
  );
  fireEvent.click(screen.getByRole('button', { name: 'Keep the receipt date' }));
  expect(screen.getByLabelText('Date', { exact: true })).toHaveProperty('value', '2025-11-01');
  expect(screen.queryByRole('alert')).toBeNull();
  expect(onSave).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole('button', { name: 'Save changes' }));
  await waitFor(() => expect(onSave).toHaveBeenCalledOnce());
  expect(onSave.mock.calls[0][0].date).toBe('2025-11-01');
});
it('offers today when a receipt date could not be read', () => {
  const receipt = { ...seedReceipts()[0], dateMissing: true, status: 'review' as const };
  render(
    <ReceiptDialog
      receipt={receipt}
      catalog={CATALOG}
      today="2026-11-01"
      demo={false}
      onSave={() => {}}
      onClose={() => {}}
      onRepeat={() => {}}
    />,
  );
  expect(screen.queryByRole('button', { name: 'Keep the receipt date' })).toBeNull();
  fireEvent.click(screen.getByRole('button', { name: "Use today's date" }));
  expect(screen.getByLabelText('Date', { exact: true })).toHaveProperty('value', '2026-11-01');
});
it("uses ISO dates in the crew's zone and keeps November in different years separate", () => {
  setCrewZone('Europe/Berlin');
  expect(crewDate(new Date('2026-10-31T23:30:00Z'))).toBe('2026-11-01');
  expect(crewDate(new Date('2026-12-31T23:30:00Z'))).toBe('2027-01-01');
  setCrewZone('UTC');
  expect(crewDate(new Date('2026-10-31T23:30:00Z'))).toBe('2026-10-31');
  const old = { ...seedReceipts()[0], date: '2025-11-01' };
  const current = { ...old, id: 'current', date: '2026-11-01', total: 2894 };
  expect(shoppingMonths([old, current], '2026-11-01')).toEqual([
    '2025-11',
    '2026-06',
    '2026-07',
    '2026-08',
    '2026-09',
    '2026-10',
    '2026-11',
  ]);
  expect(summarize([old, current], '2026-11', 'EUR').total).toBe(2894);
  expect(summarize([old, current], '2025-11', 'EUR').total).toBe(old.total);
});
it('saves corrected names from review even with a missing date, wrong sum and another uncertain line', async () => {
  const original = seedReceipts()[0];
  const receipt = {
    ...original,
    status: 'review' as const,
    dateMissing: true,
    total: original.total + 40,
    revision: 1,
    items: original.items.map((i, index) => ({ ...i, uncertain: index < 2 })),
  };
  const onSave = vi.fn();
  render(
    <ReceiptDialog
      receipt={receipt}
      catalog={CATALOG}
      demo={false}
      onSave={onSave}
      onClose={() => {}}
      onRepeat={() => {}}
    />,
  );
  fireEvent.click(screen.getByRole('button', { name: 'Check items' }));
  fireEvent.change(screen.getByRole('textbox', { name: 'Item 1 name' }), {
    target: { value: 'Oat flakes' },
  });
  expect(screen.getByRole('combobox', { name: 'Item 1 product type' })).toHaveProperty(
    'value',
    'Oat flakes',
  );
  fireEvent.click(screen.getByRole('button', { name: 'Save changes' }));
  await waitFor(() => expect(onSave).toHaveBeenCalledOnce());
  const [saved, , allowIncomplete] = onSave.mock.calls[0];
  expect(allowIncomplete).toBe(true);
  expect(saved.status).toBe('review');
  expect(saved.dateMissing).toBe(true);
  expect(saved.items[0].name).toBe('Oat flakes');
  expect(saved.items[0].productName).toBe('Oat flakes');
  expect(saved.items[1].uncertain).toBe(true);
  expect(saved.total).toBe(receipt.total);
  expect(screen.getByRole('status').textContent).toContain('Fixes saved');
});
it('keeps strict validation for a purchase that is already included in expenses', () => {
  const original = seedReceipts()[0];
  const onSave = vi.fn();
  render(
    <ReceiptDialog
      receipt={{ ...original, total: original.total + 40 }}
      catalog={CATALOG}
      demo={false}
      onSave={onSave}
      onClose={() => {}}
      onRepeat={() => {}}
    />,
  );
  fireEvent.click(screen.getByRole('button', { name: 'Fix' }));
  fireEvent.click(screen.getByRole('button', { name: 'Save changes' }));
  expect(onSave).not.toHaveBeenCalled();
  expect(screen.getByRole('alert').textContent).toContain('add up to the receipt total');
});
it('saves corrected original text and a new product type instead of retaining the wrong link', async () => {
  const receipt = { ...seedReceipts()[0], revision: 1 };
  const onSave = vi.fn();
  render(
    <ReceiptDialog
      receipt={receipt}
      catalog={CATALOG}
      demo={false}
      onSave={onSave}
      onClose={() => {}}
      onRepeat={() => {}}
    />,
  );
  fireEvent.click(screen.getByRole('button', { name: 'Fix' }));
  fireEvent.change(screen.getByRole('textbox', { name: 'Item 1 name' }), {
    target: { value: 'Oat flakes, rolled' },
  });
  fireEvent.change(screen.getByRole('textbox', { name: 'Item 1 original text' }), {
    target: { value: 'ROLLED OATS 500G' },
  });
  fireEvent.change(screen.getByRole('combobox', { name: 'Item 1 product type' }), {
    target: { value: 'Oat flakes' },
  });
  fireEvent.click(screen.getByRole('button', { name: 'Save changes' }));
  await waitFor(() => expect(onSave).toHaveBeenCalledOnce());
  const item = onSave.mock.calls[0][0].items[0];
  expect(item.productName).toBe('Oat flakes');
  expect(item.productId).toBe('oats');
  expect(item.productId).not.toBe(receipt.items[0].productId);
  expect(item.rawName).toBe('ROLLED OATS 500G');
  expect(item.amount).toBe(receipt.items[0].amount);
});
it('requires an explicit date when the photographed receipt has no visible date', () => {
  const receipt = { ...seedReceipts()[0], dateMissing: true };
  render(
    <ReceiptDialog
      receipt={receipt}
      catalog={CATALOG}
      demo={false}
      onSave={() => {}}
      onClose={() => {}}
      onRepeat={() => {}}
    />,
  );
  expect(screen.getByText(/No date/)).toBeTruthy();
  fireEvent.click(screen.getByRole('button', { name: 'Fix' }));
  expect(screen.getByLabelText('Date', { exact: true })).toHaveProperty('value', '');
});
it('uses the correct minor units for travel currencies', () => {
  expect(currencyScale('EUR')).toBe(100);
  expect(currencyScale('JPY')).toBe(1);
  expect(currencyScale('KWD')).toBe(1000);
  expect(parseMoney('1200', 'JPY')).toBe(1200);
  expect(parseMoney('1.234', 'KWD')).toBe(1234);
  expect(parseMoney('1.234', 'EUR')).toBe(null);
  expect(money(1200, 'JPY')).toContain('1');
  expect(() => money(10, 'EU')).not.toThrow();
});

it('keeps locally queued photos and draft receipts outside the financial projection', () => {
  const booked = seedReceipts()[0];
  const account = {
    uid: 'test',
    snapshot: {
      revision: 1,
      today: '2026-10-31',
      list: [],
      catalog: [],
      receipts: [{ ...booked, status: 'review' as const }],
      purchases: [booked],
      imports: [],
      aiConfigured: true,
    },
    queue: [
      {
        id: 'offline-photo',
        kind: 'upload' as const,
        photos: [],
        createdAt: '2026-10-31T12:00:00Z',
      },
    ],
  };
  const projected = projectShopping(account);
  expect(projected.receipts.some((r) => r.status === 'queued')).toBe(true);
  expect(projected.purchases).toEqual([booked]);
  expect(summarize(projected.purchases, booked.date.slice(0, 7), 'EUR').total).toBe(booked.total);
  expect(parseMoney('-0,50')).toBe(-50);
  expect(parseMoney('-999999999')).toBe(null);
});

it('does not save a stale receipt editor after a crewmate changes the source', () => {
  const original = { ...seedReceipts()[0], revision: 1 };
  const onSave = vi.fn();
  const onClose = vi.fn();
  const view = render(
    <ReceiptDialog
      receipt={original}
      catalog={CATALOG}
      demo={false}
      onSave={onSave}
      onClose={onClose}
      onRepeat={() => {}}
    />,
  );
  fireEvent.click(screen.getByRole('button', { name: 'Fix' }));
  fireEvent.change(screen.getByRole('textbox', { name: 'Item 1 name' }), {
    target: { value: 'Cat food (fixed)' },
  });
  view.rerender(
    <ReceiptDialog
      receipt={{ ...original, revision: 2 }}
      catalog={CATALOG}
      demo={false}
      onSave={onSave}
      onClose={onClose}
      onRepeat={() => {}}
    />,
  );
  fireEvent.click(screen.getByRole('button', { name: 'Save changes' }));
  expect(onSave).not.toHaveBeenCalled();
  expect(screen.getByRole('alert').textContent).toContain('changed this receipt on another device');
});

it('shows normalized and original item names and never fabricates a source photo in the real view', () => {
  const receipt = {
    ...seedReceipts()[0],
    photoExpiresAt: '2099-01-01T00:00:00Z',
    photos: [],
    photoError: 'Offline storage problem',
  };
  render(
    <ReceiptDialog
      receipt={receipt}
      catalog={CATALOG}
      demo={false}
      onSave={() => {}}
      onClose={() => {}}
      onRepeat={() => {}}
    />,
  );
  expect(screen.getByText('Cat food')).toBeTruthy();
  expect(screen.getByText(receipt.items[0].rawName)).toBeTruthy();
  expect(screen.getByText('Offline storage problem')).toBeTruthy();
  expect(screen.queryByRole('img')).toBe(null);
});
