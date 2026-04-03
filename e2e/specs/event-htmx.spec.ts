import { test, expect } from '@playwright/test';

test.describe('event form htmx interactions', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/dashboard/events/new');
  });

  test('clicking Add Reminder inserts one row', async ({ page }) => {
    await page.click('button:has-text("+ Add Reminder")');
    await page.waitForSelector('.reminder-row');
    await expect(page.locator('.reminder-row')).toHaveCount(1);
  });

  test('clicking Add Reminder twice inserts two rows', async ({ page }) => {
    await page.click('button:has-text("+ Add Reminder")');
    await page.waitForSelector('.reminder-row');
    await page.click('button:has-text("+ Add Reminder")');
    await expect(page.locator('.reminder-row')).toHaveCount(2);
  });

  test('clicking Remove deletes the row', async ({ page }) => {
    await page.click('button:has-text("+ Add Reminder")');
    await page.waitForSelector('.reminder-row');
    await expect(page.locator('.reminder-row')).toHaveCount(1);
    await page.click('.reminder-row button:has-text("Remove")');
    await expect(page.locator('.reminder-row')).toHaveCount(0);
  });

  test('defaultNoon corrects midnight datetime to noon', async ({ page }) => {
    // Insert a reminder row first
    await page.click('button:has-text("+ Add Reminder")');
    await page.waitForSelector('.reminder-row input[name="reminder_datetime"]');

    // Set value to midnight via JS and dispatch change event
    await page.evaluate(() => {
      const el = document.querySelector('input[name="reminder_datetime"]') as HTMLInputElement;
      el.value = '2026-06-15T00:00';
      el.dispatchEvent(new Event('change', { bubbles: true }));
    });

    await expect(page.locator('input[name="reminder_datetime"]')).toHaveValue('2026-06-15T12:00');
  });
});
