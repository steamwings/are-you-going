import { test, expect } from '../fixtures';
import { insertOptOut, removeOptOut } from '../helpers/db';

const TS = Date.now() % 7000 + 1000;
const PHONE_OPTED_IN    = `202555${TS}`;      // opts in on form
const PHONE_NOT_OPTED   = `202555${TS + 1}`;  // does not opt in
const PHONE_PRIOR_OUT   = `202555${TS + 2}`;  // prior opt-out, opts in on form

async function submitRsvp(page: any, slug: string, phone: string, optIn = false) {
  await page.goto(`/e/${slug}`);
  await page.fill('input[name="name"]', 'SMS Test');
  await page.fill('input[name="phone_number"]', phone);
  await page.fill('input[name="party_size"]', '1');
  if (optIn) {
    await page.check('input[name="sms_opt_in"]');
  }
  await page.click('button[type="submit"]');
  await page.waitForURL(/\/e\/.+\/thanks/);
}

test.describe('SMS opt-in', () => {
  test('thanks page is clean after opt-in submission', async ({ page, fixtureSlug }) => {
    await submitRsvp(page, fixtureSlug, PHONE_OPTED_IN, /* optIn= */ true);
    await expect(page.locator('h1')).toContainText('Thanks for your response!');
    // No re-enable prompt or opt-out messaging
    await expect(page.locator('text=previously opted out')).not.toBeVisible();
    await expect(page.locator('text=Enable SMS')).not.toBeVisible();
  });

  test('thanks page is clean without opting in', async ({ page, fixtureSlug }) => {
    await submitRsvp(page, fixtureSlug, PHONE_NOT_OPTED, /* optIn= */ false);
    await expect(page.locator('h1')).toContainText('Thanks for your response!');
    await expect(page.locator('text=previously opted out')).not.toBeVisible();
    await expect(page.locator('text=Enable SMS')).not.toBeVisible();
  });

  test('opting in with a prior opt-out record succeeds and lands on thanks', async ({ page, fixtureSlug }) => {
    insertOptOut(`+1${PHONE_PRIOR_OUT}`);
    try {
      await submitRsvp(page, fixtureSlug, PHONE_PRIOR_OUT, /* optIn= */ true);
      await expect(page.locator('h1')).toContainText('Thanks for your response!');
    } finally {
      removeOptOut(`+1${PHONE_PRIOR_OUT}`);
    }
  });
});
