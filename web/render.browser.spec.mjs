import { readFile } from 'node:fs/promises';
import { expect, test } from '@playwright/test';

test('renders, plays, and exports audible audio in Chromium', async ({ page }) => {
  const pageErrors = [];
  page.on('pageerror', error => pageErrors.push(error.message));
  await page.route(/https:\/\/(fonts\.googleapis|fonts\.gstatic)\.com\//, route => route.abort());
  await page.goto('/');
  await expect(page.locator('#status')).toContainText('Engine ready');

  await page.getByLabel('Phrase length').selectOption('1');
  await page.getByRole('button', { name: 'Render passage' }).click();
  await expect(page.locator('#status')).toContainText('Render complete', { timeout: 30000 });
  await expect(page.getByRole('button', { name: 'Play rendered passage' })).toBeEnabled();
  await expect(page.getByRole('button', { name: 'Download WAV' })).toBeEnabled();

  await page.getByRole('button', { name: 'Play rendered passage' }).click();
  await expect(page.getByRole('button', { name: 'Pause rendered passage' })).toBeVisible();
  await page.getByRole('button', { name: 'Pause rendered passage' }).click();
  await expect(page.getByRole('button', { name: 'Play rendered passage' })).toBeVisible();

  const downloadPromise = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Download WAV' }).click();
  const download = await downloadPromise;
  expect(download.suggestedFilename()).toMatch(/^obelisk-m0-.*\.wav$/);
  const wav = await readFile(await download.path());
  expect(wav.toString('ascii', 0, 4)).toBe('RIFF');
  expect(wav.toString('ascii', 8, 12)).toBe('WAVE');
  expect(wav.readUInt16LE(20)).toBe(1);
  expect(wav.readUInt16LE(22)).toBe(2);
  expect(wav.readUInt32LE(24)).toBe(48000);
  expect(wav.readUInt16LE(34)).toBe(16);
  expect(wav.readUInt32LE(40)).toBe(wav.length - 44);
  let peak = 0;
  for (let offset = 44; offset < wav.length; offset += 2) {
    peak = Math.max(peak, Math.abs(wav.readInt16LE(offset)));
  }
  expect(peak).toBeGreaterThan(100);
  expect(pageErrors).toEqual([]);
});
