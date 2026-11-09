import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import tailwind from '@tailwindcss/vite';

export default defineConfig({
  plugins: [react(), tailwind()],
  server: {
    // `npm run dev` talks to a locally running juggle-server.
    proxy: Object.fromEntries(
      ['/juggle-crews', '/juggle-routines', '/juggle-shopping', '/docs', '/openapi.json'].map(
        (path) => [path, 'http://127.0.0.1:8080'],
      ),
    ),
  },
  test: {
    environment: 'jsdom',
    // Node 25+ ships its own localStorage, which shadows jsdom's.
    execArgv: ['--no-experimental-webstorage'],
    include: ['src/**/*.test.ts', 'src/**/*.test.tsx'],
  },
});
