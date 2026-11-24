import '@kaijukit/ui/styles.css';
import '@kaijukit/editor/styles.css';
import './app.css';
import { ToastProvider } from '@kaijukit/ui';
import { MutationCache, QueryCache, QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { BrowserRouter } from 'react-router-dom';
import { ApiError } from './api';
import { App } from './App';
import { AuthProvider } from './auth';
import { store } from './storage';
import { applyTheme, type Theme } from './theme';

applyTheme((store.get('kk.theme') as Theme | null) ?? 'auto');

let logout = () => {};
const onError = (e: unknown) => e instanceof ApiError && e.status === 401 && logout();
const client = new QueryClient({
  defaultOptions: { queries: { retry: (n, e) => !(e instanceof ApiError && e.status !== 0 && e.status < 500) && n < 2, staleTime: 15_000 } },
  queryCache: new QueryCache({ onError }),
  mutationCache: new MutationCache({ onError }),
});

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <QueryClientProvider client={client}>
      <AuthProvider onLogout={(f) => (logout = f)}>
        <ToastProvider>
          <BrowserRouter future={{ v7_startTransition: true, v7_relativeSplatPath: true }}>
            <App />
          </BrowserRouter>
        </ToastProvider>
      </AuthProvider>
    </QueryClientProvider>
  </StrictMode>,
);
