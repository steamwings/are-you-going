import { test, expect } from '../fixtures';
import { insertOptOut, removeOptOut } from '../helpers/db';

// Unique 10-digit US phones per run. Cap at 7996 so +0..+3 stays 4 digits.
// Range 1000–7999 avoids collisions with fixed fixture phones (0100–0105).
const TS = Date.now() % 7000 + 1000;
const PHONE_NORMAL      = `202555${TS}`;      // no prior opt-out
const PHONE_OPTED_OUT   = `202555${TS + 1}`;  // has a prior opt-out, opts in
const PHONE_FORM_OPTOUT = `202555${TS + 2}`;  // has a prior opt-out, opts out on form
const PHONE_REENABLE    = `202555${TS + 3}`;  // clicks Enable

async function submitRsvp(page: any, slug: string, phone: string, optOut = false) {
  await page.goto(`/e/${slug}`);
  await page.fill('input[name="name"]', 'SMS Test');
  await page.fill('input[name="phone_number"]', phone);
  await page.fill('input[name="party_size"]', '1');
  if (optOut) {
    await page.check('input[name="sms_opt_out"]');
  }
  await page.click('button[type="submit"]');
  await page.waitForURL(/\/e\/.+\/thanks/);
}

test.describe('SMS re-enable prompt', () => {
  test('no prompt for a normal submission with no prior opt-out', async ({ page, fixtureSlug }) => {
    await submitRsvp(page, fixtureSlug, PHONE_NORMAL);
    await expect(page.locator('text=previously opted out')).not.toBeVisible();
  });

  test('prompt appears when submitting with opt-in after a prior STOP', async ({ page, fixtureSlug }) => {
    insertOptOut(`+1${PHONE_OPTED_OUT}`);
    try {
      await submitRsvp(page, fixtureSlug, PHONE_OPTED_OUT);
      await expect(page.locator('text=previously opted out')).toBeVisible();
      await expect(page.locator('button:has-text("Enable SMS notifications")')).toBeVisible();
    } finally {
      removeOptOut(`+1${PHONE_OPTED_OUT}`);
    }
  });

  test('no prompt when user checks the opt-out box on the form', async ({ page, fixtureSlug }) => {
    insertOptOut(`+1${PHONE_FORM_OPTOUT}`);
    try {
      await submitRsvp(page, fixtureSlug, PHONE_FORM_OPTOUT, /* optOut= */ true);
      await expect(page.locator('text=previously opted out')).not.toBeVisible();
    } finally {
      removeOptOut(`+1${PHONE_FORM_OPTOUT}`);
    }
  });

  test('clicking Enable removes the opt-out and returns to a clean thanks page', async ({ page, fixtureSlug }) => {
    insertOptOut(`+1${PHONE_REENABLE}`);
    await submitRsvp(page, fixtureSlug, PHONE_REENABLE);

    // Prompt should be visible before enabling
    await expect(page.locator('button:has-text("Enable SMS notifications")')).toBeVisible();

    await page.click('button:has-text("Enable SMS notifications")');

    // Server removes opt-out and redirects to thanks with no query params
    await page.waitForURL(`**/e/${fixtureSlug}/thanks**`);
    await expect(page.locator('h1')).toContainText('Thanks for your response!');
    await expect(page.locator('text=previously opted out')).not.toBeVisible();
  });
});
