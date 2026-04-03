import { test, expect } from '../fixtures';

// Use a timestamp-derived suffix to avoid duplicate-phone errors on dev DB reruns.
// Range 1000-8999 avoids collisions with fixed fixture phones (0100-0105).
const SUBMIT_PHONE = `202555${String(Date.now() % 8000 + 1000)}`;

test.describe('RSVP submit', () => {
  test('form shows event name', async ({ page, fixtureSlug }) => {
    await page.goto(`/e/${fixtureSlug}`);
    await expect(page.locator('h1')).toContainText('E2E Fixture Event');
  });

  test('valid RSVP submission lands on thanks page', async ({ page, fixtureSlug }) => {
    await page.goto(`/e/${fixtureSlug}`);
    await page.fill('input[name="name"]', 'Submit Tester');
    await page.fill('input[name="phone_number"]', SUBMIT_PHONE);
    await page.fill('input[name="party_size"]', '1');
    await page.click('button[type="submit"]');
    await page.waitForURL(`**/e/${fixtureSlug}/thanks**`);
    await expect(page.locator('h1')).toContainText('Thanks for your response!');
  });

  test('thanks page has edit link', async ({ page, fixtureSlug }) => {
    await page.goto(`/e/${fixtureSlug}/thanks`);
    await expect(page.locator('a[href*="/edit"]')).toBeVisible();
  });

  test('duplicate phone number shows error', async ({ page, fixtureSlug }) => {
    await page.goto(`/e/${fixtureSlug}`);
    await page.fill('input[name="name"]', 'Duplicate');
    await page.fill('input[name="phone_number"]', SUBMIT_PHONE);
    await page.fill('input[name="party_size"]', '1');
    await page.click('button[type="submit"]');
    await expect(page.locator('mark')).toContainText('already exists');
  });

  test('invalid phone number shows error', async ({ page, fixtureSlug }) => {
    await page.goto(`/e/${fixtureSlug}`);
    await page.fill('input[name="name"]', 'Bad Phone');
    await page.fill('input[name="phone_number"]', 'notaphone');
    await page.fill('input[name="party_size"]', '1');
    await page.click('button[type="submit"]');
    await expect(page.locator('mark')).toContainText('Invalid phone');
  });

  test('empty phone is invalid per browser validation', async ({ page, fixtureSlug }) => {
    await page.goto(`/e/${fixtureSlug}`);
    const isValid = await page.evaluate(() => {
      const el = document.querySelector('#phone_number') as HTMLInputElement;
      return el.validity.valid;
    });
    expect(isValid).toBe(false);
  });

  test('opt-in checkbox is unchecked by default', async ({ page, fixtureSlug }) => {
    await page.goto(`/e/${fixtureSlug}`);
    await expect(page.locator('input[name="sms_opt_in"]')).not.toBeChecked();
  });

  test('404 for unknown event slug', async ({ page }) => {
    const response = await page.goto('/e/does-not-exist-xyz');
    expect(response?.status()).toBe(404);
  });

  test('trailing slash resolves to same event page', async ({ page, fixtureSlug }) => {
    await page.goto(`/e/${fixtureSlug}/`);
    await expect(page.locator('h1')).toContainText('E2E Fixture Event');
  });
});
