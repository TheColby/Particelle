import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './web',
  testMatch: '**/*.browser.spec.mjs',
  workers: 1,
  timeout: 45000,
  use: {
    baseURL: 'http://127.0.0.1:8765',
    browserName: 'chromium',
    channel: process.env.PLAYWRIGHT_CHROME_CHANNEL || undefined,
    screenshot: 'only-on-failure',
    trace: 'retain-on-failure',
  },
  webServer: {
    command: 'python3 -m http.server 8765 --bind 127.0.0.1 --directory web',
    url: 'http://127.0.0.1:8765/',
    reuseExistingServer: !process.env.CI,
    timeout: 30000,
  },
});
