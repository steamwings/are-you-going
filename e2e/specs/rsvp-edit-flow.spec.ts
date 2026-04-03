import { test, expect } from '../fixtures';
import { getMagicLinkToken } from '../helpers/db';

const EDIT_PHONE = '+12025550105';
const EDIT_PHONE_RAW = '2025550105';

test.describe('RSVP edit flow', () => {
  // Create an RSVP for the edit flow tests before any test in this suite runs
  test.beforeAll(async ({ browser }) => {
    const ctx  = await browser.newContext();
    const page = await ctx.newPage();
    const slug = 'e2e-fixture';
    await page.goto(`http://localhost:5150/e/${slug}`);
    await page.fill('input[name="name"]', 'Edit Flow Tester');
    await page.fill('input[name="phone_number"]', EDIT_PHONE_RAW);
    await page.fill('input[name="party_size"]', '1');
    await page.click('button[type="submit"]');
    // Either lands on /thanks (new RSVP) or stays on form with duplicate error — both OK
    await page.waitForLoadState('networkidle');
    await ctx.close();
  });

  test('edit phone form renders', async ({ page, fixtureSlug }) => {
    await page.goto(`/e/${fixtureSlug}/edit`);
    await expect(page.locator('h1')).toContainText('Edit your RSVP');
  });

  test('unknown phone shows error', async ({ page, fixtureSlug }) => {
    await page.goto(`/e/${fixtureSlug}/edit`);
    // Valid format but no RSVP registered for this number
    await page.fill('input[name="phone_number"]', '2025550199');
    await page.click('button[type="submit"]');
    await expect(page.locator('mark')).toContainText('No RSVP found');
  });

  test('valid phone shows magic link sent page', async ({ page, fixtureSlug }) => {
    await page.goto(`/e/${fixtureSlug}/edit`);
    await page.fill('input[name="phone_number"]', EDIT_PHONE_RAW);
    await page.click('button[type="submit"]');
    await expect(page.locator('h1')).toContainText('Check your phone!');
    // URL stays on the slug context, not redirected away
    await expect(page).toHaveURL(new RegExp(`/e/${fixtureSlug}/edit`));
  });

  test('edit form is pre-filled with existing RSVP data', async ({ page, fixtureSlug }) => {
    // Request magic link first
    await page.goto(`/e/${fixtureSlug}/edit`);
    await page.fill('input[name="phone_number"]', EDIT_PHONE_RAW);
    await page.click('button[type="submit"]');
    await expect(page.locator('h1')).toContainText('Check your phone!');

    // Read token from DB
    const token = getMagicLinkToken(EDIT_PHONE, fixtureSlug);
    await page.goto(`/e/${fixtureSlug}/edit/${token}`);

    await expect(page.locator('#phone_number')).toHaveValue(EDIT_PHONE);
    // Name may have been updated by prior runs; just verify it's pre-populated
    await expect(page.locator('#name')).toHaveValue(/Edit Flow Tester/);
  });

  test('invalid token returns not-found', async ({ page, fixtureSlug }) => {
    const response = await page.goto(`/e/${fixtureSlug}/edit/not-a-real-token`);
    expect(response?.status()).toBe(404);
  });

  test('updating RSVP redirects to thanks page', async ({ page, fixtureSlug }) => {
    // Request magic link
    await page.goto(`/e/${fixtureSlug}/edit`);
    await page.fill('input[name="phone_number"]', EDIT_PHONE_RAW);
    await page.click('button[type="submit"]');
    await expect(page.locator('h1')).toContainText('Check your phone!');

    const token = getMagicLinkToken(EDIT_PHONE, fixtureSlug);
    await page.goto(`/e/${fixtureSlug}/edit/${token}`);

    await page.fill('#name', 'Edit Flow Tester Updated');
    await page.fill('input[name="party_size"]', '3');
    await page.click('button[type="submit"]');
    await page.waitForURL(`**/e/${fixtureSlug}/thanks**`);
    await expect(page.locator('h1')).toContainText('Thanks for your response!');
  });
});
