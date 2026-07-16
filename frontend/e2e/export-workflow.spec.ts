import { expect, test } from '@playwright/test';

test('connects to the configured Oracle and enables export configuration', async ({ page }) => {
  const consoleErrors: string[] = [];
  page.on('console', (message) => {
    if (message.type() === 'error') {
      consoleErrors.push(message.text());
    }
  });

  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Export the Oracle you can inspect, move, and keep.' })).toBeVisible();
  await page.getByRole('button', { name: 'Test connection' }).click();
  await expect(page.getByText('Connected', { exact: true })).toBeVisible();
  await expect(page.locator('#collection option')).not.toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Create export' })).toBeEnabled();
  await expect(page.getByText(/records across/)).toBeVisible();
  await expect.poll(() => consoleErrors).toEqual([]);
  await page.screenshot({ path: '../artifacts/issue-1-connected-desktop.png', fullPage: true });
});

test('keeps the connection flow usable at a narrow viewport', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await expect(page.getByLabel('Oracle URL')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Test connection' })).toBeVisible();
  await expect(page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).resolves.toBe(true);
  await page.screenshot({ path: '../artifacts/issue-1-mobile.png', fullPage: true });
});
