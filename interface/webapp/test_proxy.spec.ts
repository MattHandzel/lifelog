import { test, expect } from '@playwright/test';
test('verify media proxy is reachable', async ({ page }) => {
  const response = await page.goto('http://localhost:5173/media/test-hash');
  console.log('[TEST] Proxy status:', response?.status());
  expect(response?.status()).toBe(404);
});
