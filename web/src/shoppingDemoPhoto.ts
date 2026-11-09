import type { Receipt } from './shoppingModels';
export function demoPhoto(receipt: Receipt): string {
  const escape = (s: string) =>
    s.replace(
      /[<>&"']/g,
      (c) => ({ '<': '&lt;', '>': '&gt;', '&': '&amp;', '"': '&quot;', "'": '&apos;' })[c]!,
    );
  const lines = receipt.items
    .map(
      (item, index) =>
        `<text x="24" y="${136 + index * 42}" font-size="11">${escape(item.rawName.slice(0, 29))}</text><text x="276" y="${153 + index * 42}" font-size="12" text-anchor="end">${((item.amount + (receipt.status === 'review' && index === 2 ? 170 : 0)) / 100).toFixed(2)}</text>`,
    )
    .join('');
  const height = 240 + receipt.items.length * 42;
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="300" height="${height}" viewBox="0 0 300 ${height}"><rect width="300" height="${height}" fill="#faf9f4"/><g fill="#3d4148" font-family="monospace"><text x="150" y="35" text-anchor="middle" font-size="22" font-weight="bold">${escape(receipt.merchant)}</text><text x="150" y="58" text-anchor="middle" font-size="10">DEMO RECEIPT</text><text x="24" y="87" font-size="11">${receipt.date} · 18:42</text><path d="M24 104H276" stroke="#a4a5a6" stroke-dasharray="4 3"/>${lines}<path d="M24 ${height - 92}H276" stroke="#a4a5a6" stroke-dasharray="4 3"/><text x="24" y="${height - 62}" font-size="16" font-weight="bold">TOTAL</text><text x="276" y="${height - 62}" text-anchor="end" font-size="16" font-weight="bold">${(receipt.total / 100).toFixed(2)} ${escape(receipt.currency)}</text><text x="150" y="${height - 25}" text-anchor="middle" font-size="10">Made-up data · not a real receipt</text></g></svg>`;
  return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
}
