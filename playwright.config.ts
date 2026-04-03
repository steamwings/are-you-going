import { defineConfig, devices } from '@playwright/test';
import path from 'path';
import fs from 'fs';

// Load .env.dev for local development (won't override vars already in the environment)
const envDevPath = path.join(__dirname, '.env.dev');
if (fs.existsSync(envDevPath)) {
  for (const line of fs.readFileSync(envDevPath, 'utf-8').split('\n')) {
    const m = line.match(/^([A-Z_][A-Z0-9_]*)=(.*)$/);
    if (m && !process.env[m[1]]) process.env[m[1]] = m[2];
  }
}

export const STORAGE_STATE = path.join(__dirname, 'e2e/.auth/dashboard.json');
const BASE_URL = 'http://localhost:5150';

export default defineConfig({
  testDir: './e2e/specs',
  fullyParallel: false,
  workers: 1,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI
    ? [['github'], ['html', { open: 'never' }]]
    : [['list'], ['html', { open: 'on-failure' }]],

  use: {
    baseURL: BASE_URL,
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },

  globalSetup: './e2e/global.setup.ts',

  projects: [
    {
      name: 'public',
      use: { ...devices['Desktop Chrome'] },
      testMatch: ['auth.spec.ts', 'rsvp-*.spec.ts'],
    },
    {
      name: 'dashboard',
      use: { ...devices['Desktop Chrome'], storageState: STORAGE_STATE },
      testMatch: ['dashboard.spec.ts', 'event-*.spec.ts'],
    },
  ],

  webServer: {
    command: 'LOCO_ENV=test DATABASE_URL=sqlite://are-you-going_e2e.sqlite?mode=rwc DASHBOARD_PASSWORD=testpassword SMS_PROVIDER=mock BASE_URL=http://localhost:5150 cargo run',
    url: BASE_URL + '/dashboard/login',
    reuseExistingServer: !process.env.CI,
    timeout: 5 * 60 * 1000,
    stdout: 'pipe',
    stderr: 'pipe',
  },
});
