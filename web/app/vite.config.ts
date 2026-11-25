import react from '@vitejs/plugin-react';
import { defineConfig } from 'vitest/config';
import { mockApi } from '../dev/mock';

// KK_BACKEND=http://localhost:8080 proxies to the real server; otherwise the in-memory mock answers.
const backend = process.env.KK_BACKEND;

export default defineConfig({
  plugins: [react(), ...(backend ? [] : [mockApi()])],
  server: { port: 5173, proxy: backend ? { '/wiki/api': backend, '/wiki/rest': backend, '/wiki/download': backend } : undefined },
  test: { environment: 'jsdom' },
});
