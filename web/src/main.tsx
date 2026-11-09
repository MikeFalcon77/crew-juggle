import React from 'react';
import { createRoot } from 'react-dom/client';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import App from './App';
import './styles.css';
import './theme.css';
const client = new QueryClient({
  defaultOptions: {
    queries: { retry: false, refetchOnWindowFocus: 'always' },
    mutations: { retry: false },
  },
});
createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <QueryClientProvider client={client}>
      <App />
    </QueryClientProvider>
  </React.StrictMode>,
);
if ('serviceWorker' in navigator && import.meta.env.PROD)
  void navigator.serviceWorker.register('/sw.js');
