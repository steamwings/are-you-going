import { test, expect } from '../fixtures';

async function fillAndBlur(page: import('@playwright/test').Page, phone: string) {
  // Register listener before fill — fill() may trigger blur on a prior focused element,
  // causing htmx to fire the format request earlier than expected.
  const responsePromise = page.waitForResponse('**/e/phone/format*');
  await page.locator('#phone_number').fill(phone);
  await page.locator('#phone_number').blur();
  await responsePromise;
}

test.describe('RSVP phone auto-format', () => {
  test.beforeEach(async ({ page, fixtureSlug }) => {
    await page.goto(`/e/${fixtureSlug}`);
  });

  test('10-digit number formats on blur', async ({ page }) => {
    await fillAndBlur(page, '2025550102');
    await expect(page.locator('#phone_number')).toHaveValue('+1 (202) 555-0102');
  });

  test('E.164 number formats on blur', async ({ page }) => {
    await fillAndBlur(page, '+12025550103');
    await expect(page.locator('#phone_number')).toHaveValue('+1 (202) 555-0103');
  });

  test('invalid number is unchanged after blur', async ({ page }) => {
    // Invalid numbers don't trigger a format response — htmx fires but server returns unchanged
    const responsePromise = page.waitForResponse('**/e/phone/format*');
    await page.locator('#phone_number').fill('12345');
    await page.locator('#phone_number').blur();
    await responsePromise;
    await expect(page.locator('#phone_number')).toHaveValue('12345');
  });

  test('edit phone form also reformats on blur', async ({ page, fixtureSlug }) => {
    await page.goto(`/e/${fixtureSlug}/edit`);
    const responsePromise = page.waitForResponse('**/e/phone/format*');
    await page.locator('#phone_number').fill('2025550104');
    await page.locator('#phone_number').blur();
    await responsePromise;
    await expect(page.locator('#phone_number')).toHaveValue('+1 (202) 555-0104');
  });
});
