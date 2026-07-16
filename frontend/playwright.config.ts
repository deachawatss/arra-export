import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  use: {
    baseURL: process.env.ARRA_EXPORT_BASE_URL ?? 'http://localhost:4788',
    viewport: { width: 1920, height: 1080 }
  },
  reporter: 'list'
});
