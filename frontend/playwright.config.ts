import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  webServer: {
    command: 'cd frontend && bun run build && cd .. && ARRA_EXPORT_PORT=4788 cargo run -p arra-export',
    cwd: '..',
    port: 4788,
    reuseExistingServer: !process.env.CI
  },
  use: {
    baseURL: process.env.ARRA_EXPORT_BASE_URL ?? 'http://localhost:4788',
    viewport: { width: 1920, height: 1080 }
  },
  reporter: 'list'
});
