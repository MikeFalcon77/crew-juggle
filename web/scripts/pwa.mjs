import { readFileSync, writeFileSync, readdirSync } from 'node:fs';
import { createHash } from 'node:crypto';
const assets = readdirSync('dist/assets')
  .sort()
  .map((name) => `/assets/${name}`);
const icons = readdirSync('dist/icons')
  .sort()
  .map((name) => `/icons/${name}`);
const shell = ['/index.html', '/icon.svg', '/manifest.webmanifest', ...icons, ...assets];
const fingerprint = createHash('sha256');
for (const path of shell) fingerprint.update(path).update(readFileSync(`dist${path}`));
const build = fingerprint.digest('hex').slice(0, 12);
writeFileSync(
  'dist/sw.js',
  readFileSync('dist/sw.js', 'utf8')
    .replace('__BUILD__', build)
    .replace('__SHELL__', JSON.stringify(shell)),
);
