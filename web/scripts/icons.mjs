// Regenerate committed PWA PNGs after editing public/icon.svg.
// Requires Playwright Chromium, or PWA_ICON_BROWSER=msedge / chrome.
import { chromium } from '@playwright/test';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';

const output = 'public/icons';
mkdirSync(output, { recursive: true });
const source = readFileSync('public/icon.svg', 'utf8');
writeFileSync(`${output}/crew-juggle.svg`, source);
// OS notification badges use only the alpha silhouette of the brand artwork.
const badge = source
  .replace(/<rect width="192" height="192" fill="#[0-9a-f]+"\s*\/>/i, '')
  .replace(/#[0-9a-f]{6}/gi, '#ffffff');
const browser = await chromium.launch({
  ...(process.env.PWA_ICON_BROWSER ? { channel: process.env.PWA_ICON_BROWSER } : {}),
});
try {
  const page = await browser.newPage({ deviceScaleFactor: 1 });
  for (const [name, size, svg, transparent] of [
    ['crew-juggle-192.png', 192, source, false],
    ['crew-juggle-512.png', 512, source, false],
    ['crew-juggle-maskable-192.png', 192, source, false],
    ['crew-juggle-maskable-512.png', 512, source, false],
    ['apple-touch-icon.png', 180, source, false],
    ['favicon-32.png', 32, source, false],
    ['notification-badge.png', 96, badge, true],
  ]) {
    await page.setViewportSize({ width: size, height: size });
    await page.setContent(
      `<style>html,body{margin:0;background:transparent}svg{display:block;width:100vw;height:100vh}</style>${svg}`,
    );
    await page.evaluate(() => new Promise((done) => requestAnimationFrame(() => done(null))));
    await page.screenshot({ path: `${output}/${name}`, omitBackground: transparent });
  }
  // Demo avatars, baked into the routines gear so a fresh demo already has faces.
  mkdirSync('../gears/juggle-routines/juggle-routines/demo-avatars', { recursive: true });
  for (const name of ['pip', 'zed', 'mochi', 'nova', 'ike']) {
    const svg = readFileSync(`public/characters/${name}.svg`, 'utf8');
    await page.setViewportSize({ width: 256, height: 256 });
    await page.setContent(
      `<style>html,body{margin:0;background:transparent}svg{display:block;width:100vw;height:100vh}</style>${svg}`,
    );
    await page.screenshot({ path: `../gears/juggle-routines/juggle-routines/demo-avatars/${name}.png` });
  }
} finally {
  await browser.close();
}
