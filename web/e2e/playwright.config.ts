import { defineConfig } from '@playwright/test';

const external = process.env.E2E_BASE_URL;

export default defineConfig({
  testDir: 'tests',
  workers: 1,
  retries: 0,
  timeout: 30_000,
  reporter: [['list']],
  use: { baseURL: external ?? 'http://localhost:5173', trace: 'retain-on-failure' },
  // Without E2E_BASE_URL the app runs against the in-memory mock; with it, against whatever serves that URL (real stack).
  webServer: external ? undefined : { command: 'pnpm --filter @kaijukit/app dev --strictPort', cwd: '..', url: 'http://localhost:5173', reuseExistingServer: true, timeout: 60_000 },
});
