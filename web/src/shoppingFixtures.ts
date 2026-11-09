// Entirely fictional test fixtures. No production modules, credentials or API calls.
import type { CategoryId, Product, Receipt, ReceiptItem } from './shoppingModels';

const product = (
  id: string,
  name: string,
  category: CategoryId,
  subcategory: string,
  variant: string,
  aliases: string[] = [],
): Product => ({ id, name, category, subcategory, variant, aliases });

export const CATALOG: Product[] = [
  product('cat-food', 'Cat food', 'pets', 'Pet food', 'Purrfect, 12 pouches', ['cat pouches']),
  product('litter', 'Cat litter', 'pets', 'Pet supplies', 'Clumping, 10 l'),
  product('oat-milk', 'Oat milk', 'food', 'Drinks', 'Oatly, 1 l', ['oat drink']),
  product('oats', 'Oat flakes', 'food', 'Bread & grains', 'Rolled oats, 500 g'),
  product('noodles', 'Instant noodles', 'food', 'Snacks', 'Spicy, 5 pack'),
  product('soap', 'Dish soap', 'home', 'Cleaning', 'Lemon, 500 ml'),
  product('resistors', 'Resistor kit', 'hobbies', 'Electronics & parts', '600 pcs'),
];

const line = (id: string, quantity: number, amount: number, unit = 'pc'): ReceiptItem => {
  const p = CATALOG.find((c) => c.id === id)!;
  return {
    id: `item-${id}`,
    productId: id,
    productName: p.name,
    name: p.name,
    rawName: p.variant.toUpperCase(),
    quantity,
    unit,
    amount,
    category: p.category,
    subcategory: p.subcategory,
  };
};

export function seedReceipts(): Receipt[] {
  const items = [
    line('cat-food', 1, 449),
    line('oat-milk', 2, 378),
    line('noodles', 1, 279),
    line('soap', 1, 189),
  ];
  return [
    {
      id: '2026-10-03-corner',
      merchant: 'Corner Market',
      date: '2026-10-03',
      currency: 'EUR',
      items,
      total: items.reduce((sum, i) => sum + i.amount, 0),
      status: 'saved',
    },
  ];
}
