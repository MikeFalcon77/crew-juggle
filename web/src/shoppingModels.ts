import { getCrewZone } from './time';
export type CategoryId =
  | 'food'
  | 'restaurants'
  | 'home'
  | 'pets'
  | 'hobbies'
  | 'clothes'
  | 'transport'
  | 'other'
  | 'health'
  | 'leisure'
  | 'services';
export const CATEGORIES: Record<CategoryId, { name: string; color: string; sub: string[] }> = {
  food: {
    name: 'Food',
    color: '#7564c7',
    sub: [
      'Dairy & eggs',
      'Fruit & vegetables',
      'Meat & fish',
      'Bread & grains',
      'Drinks',
      'Snacks',
      'Other food',
    ],
  },
  restaurants: { name: 'Eating out', color: '#d69b74', sub: ['Eating out'] },
  home: { name: 'Home', color: '#73a391', sub: ['Cleaning', 'Hygiene', 'Household'] },
  pets: { name: 'Pets', color: '#e0a35c', sub: ['Pet food', 'Pet supplies'] },
  hobbies: { name: 'Hobbies', color: '#5fa8c8', sub: ['Electronics & parts', 'Crafts'] },
  clothes: { name: 'Clothes', color: '#92a9d0', sub: ['Clothes'] },
  transport: { name: 'Transport', color: '#b39ac0', sub: ['Trips', 'Fuel'] },
  health: { name: 'Health', color: '#b589a2', sub: ['Pharmacy', 'Personal care'] },
  leisure: { name: 'Leisure', color: '#c2aa73', sub: ['Entertainment', 'Books', 'Sports'] },
  services: { name: 'Services', color: '#809fad', sub: ['Services'] },
  other: { name: 'Other', color: '#a6afbf', sub: ['Uncategorized'] },
};
export type Product = {
  id: string;
  name: string;
  category: CategoryId;
  subcategory: string;
  aliases: string[];
  variant: string;
  revision?: number;
  classified?: boolean;
  categoryOverride?: boolean;
};
export type CorrectionRule = {
  id: string;
  merchant: string;
  originals: string[];
  name: string;
  productId: string;
  productName: string;
  category: CategoryId;
  subcategory: string;
  revision: number;
};
export type ListItem = {
  id: string;
  productId?: string;
  name: string;
  quantity: string;
  category: CategoryId;
  done: boolean;
  author: string;
  revision?: number;
};
export type ReceiptItem = {
  productName?: string;
  id: string;
  productId?: string;
  name: string;
  rawName: string;
  quantity: number;
  unit: string;
  amount: number;
  category: CategoryId;
  subcategory: string;
  uncertain?: boolean;
};
export type Receipt = {
  dateMissing?: boolean;
  id: string;
  merchant: string;
  date: string;
  currency: string;
  total: number;
  items: ReceiptItem[];
  status: 'saved' | 'review' | 'queued' | 'processing' | 'failed';
  revision?: number;
  createdAt?: string;
  photoExpiresAt?: string;
  time?: string | null;
  issues?: string[];
  suspectedDuplicate?: string | null;
  photos?: string[];
  photoError?: string;
  matchedIds?: string[];
  eligibleIds?: string[];
};

/** Today in this device's zone; the server sends the crew's own date too. */
/** A date as `YYYY-MM-DD` in the crew's time zone. */
export function crewDate(at = new Date()) {
  const parts = new Intl.DateTimeFormat('en-CA', {
    timeZone: getCrewZone(),
    numberingSystem: 'latn',
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  }).formatToParts(at);
  const part = (type: Intl.DateTimeFormatPartTypes) => parts.find((p) => p.type === type)!.value;
  return `${part('year')}-${part('month')}-${part('day')}`;
}
export function photoExpired(date: string, expiresAt?: string) {
  return expiresAt
    ? Date.parse(expiresAt) <= Date.now()
    : (Date.parse(crewDate() + 'T12:00:00Z') - Date.parse(date + 'T12:00:00Z')) / 86400000 >= 90;
}
export function lastSixMonths(today = crewDate()) {
  const date = new Date(today.slice(0, 7) + '-01T12:00:00Z');
  return Array.from({ length: 6 }, (_, i) => {
    const d = new Date(date);
    d.setUTCMonth(d.getUTCMonth() - 5 + i);
    return d.toISOString().slice(0, 7);
  });
}
export function shoppingMonths(receipts: Pick<Receipt, 'date'>[], today = crewDate()) {
  return [...new Set([...lastSixMonths(today), ...receipts.map((r) => r.date.slice(0, 7))])]
    .filter((month) => /^\d{4}-(0[1-9]|1[0-2])$/.test(month))
    .sort();
}
const precision = new Map<string, number>();
export function currencyDigits(currency = 'EUR') {
  if (precision.has(currency)) return precision.get(currency)!;
  let digits = 2;
  try {
    digits =
      new Intl.NumberFormat('en-US', { style: 'currency', currency }).resolvedOptions()
        .maximumFractionDigits ?? 2;
  } catch {}
  precision.set(currency, digits);
  return digits;
}
export function currencyScale(currency = 'EUR') {
  return 10 ** currencyDigits(currency);
}
export function decimalAmount(minor: number, currency = 'EUR') {
  return (minor / currencyScale(currency)).toFixed(currencyDigits(currency));
}
export function money(minor: number, currency = 'EUR') {
  try {
    return new Intl.NumberFormat('en-US', { style: 'currency', currency }).format(
      minor / currencyScale(currency),
    );
  } catch {
    return `${(minor / 100).toFixed(2)} ${currency}`;
  }
}

export function monthLabel(month: string, short = false) {
  return new Intl.DateTimeFormat('en-US', {
    month: short ? 'short' : 'long',
    ...(short ? {} : { year: 'numeric' as const }),
    timeZone: 'UTC',
  }).format(new Date(`${month}-01T12:00:00Z`));
}

export function dateLabel(date: string) {
  return new Intl.DateTimeFormat('en-US', {
    day: 'numeric',
    month: 'long',
    timeZone: 'UTC',
  }).format(new Date(`${date}T12:00:00Z`));
}

export function normalize(text: string) {
  return text.toLocaleLowerCase('en').replace(/\s+/g, ' ').trim();
}

export function findProduct(text: string, catalog: Product[] = []) {
  const query = normalize(text);
  return catalog.find((p) => [p.name, ...p.aliases].some((alias) => normalize(alias) === query));
}

export function suggestions(text: string, catalog: Product[] = []) {
  const query = normalize(text);
  return catalog
    .filter(
      (p) => !query || [p.name, p.variant, ...p.aliases].some((s) => normalize(s).includes(query)),
    )
    .slice(0, 6);
}

export function parseList(text: string, catalog: Product[] = []): ListItem[] {
  return text
    .replace(/(\d),(\d)/g, '$1.$2')
    .split(/\n|;|,|\s+and\s+/)
    .map((line) =>
      line
        .replace(/^\s*(?:[-•*]|\d+[.)])\s+/, '')
        .replace(/^(?:we need|need|buy|get)\s+/i, '')
        .trim(),
    )
    .filter(Boolean)
    .map((line) => {
      const suffix = line.match(/\s+(\d+(?:\.\d+)?(?:\s*(?:kg|g|l|ml|pcs?|packs?))?)$/i);
      const prefix = line.match(/^(\d+(?:\.\d+)?(?:\s*(?:kg|g|l|ml|pcs?))?)\s+(.+)$/i);
      const name = suffix ? line.slice(0, -suffix[0].length) : prefix ? prefix[2] : line;
      const known = findProduct(name, catalog);
      return {
        id: crypto.randomUUID(),
        productId: known?.id,
        name: known?.name || name,
        quantity: suffix?.[1] || prefix?.[1] || '1',
        category: known?.category || 'other',
        done: false,
        author: 'You',
      };
    });
}

export function summarize(receipts: Receipt[], month: string, currency: string) {
  const selected = receipts.filter(
    (r) => r.status === 'saved' && r.date.startsWith(month) && r.currency === currency,
  );
  const byCategory = Object.entries(CATEGORIES)
    .map(([id, category]) => ({
      id: id as CategoryId,
      ...category,
      amount: selected
        .flatMap((r) => r.items)
        .filter((item) => item.category === id)
        .reduce((sum, item) => sum + item.amount, 0),
    }))
    .filter((c) => c.amount !== 0)
    .sort((a, b) => b.amount - a.amount);
  const days = [...new Set(selected.map((r) => r.date))].sort().map((date) => ({
    date,
    amount: selected.filter((r) => r.date === date).reduce((sum, r) => sum + r.total, 0),
  }));
  return {
    receipts: selected,
    total: selected.reduce((sum, r) => sum + r.total, 0),
    byCategory,
    turnover: byCategory.reduce((sum, c) => sum + Math.abs(c.amount), 0),
    hasRefunds: selected.some((r) => r.total < 0),
    days,
  };
}

export function parseMoney(text: string, currency = 'EUR'): number | null {
  const cleaned = text.trim().replace(',', '.');
  const digits = currencyDigits(currency);
  if (cleaned.length > 24 || !/^-?\d+(?:\.\d+)?$/.test(cleaned)) return null;
  const negative = cleaned.startsWith('-');
  const [whole, fraction = ''] = (negative ? cleaned.slice(1) : cleaned).split('.');
  if (fraction.length > digits) return null;
  const minor =
    BigInt(whole) * BigInt(currencyScale(currency)) + BigInt(fraction.padEnd(digits, '0') || '0');
  if (minor > 100000000n) return null;
  return Number(negative ? -minor : minor);
}
