import { test, expect } from '../fixtures';

test.describe('dashboard', () => {
  test('fixture event appears in index table with RSVP count', async ({ page, fixtureSlug }) => {
    await page.goto('/dashboard');
    const row = page.locator('tr', { hasText: 'E2E Fixture Event' });
    await expect(row).toBeVisible();
    const countText = await row.locator('td').nth(2).textContent();
    expect(parseInt(countText ?? '0')).toBeGreaterThanOrEqual(1);
  });

  test('event name links to detail page', async ({ page, fixtureEventId }) => {
    await page.goto('/dashboard');
    await page.locator('a', { hasText: 'E2E Fixture Event' }).click();
    await expect(page).toHaveURL(new RegExp(`/dashboard/events/${fixtureEventId}$`));
  });

  test('detail page shows public link', async ({ page, fixtureEventId }) => {
    await page.goto(`/dashboard/events/${fixtureEventId}`);
    const link = page.locator('a[href*="/e/e2e-fixture"]');
    await expect(link).toBeVisible();
  });

  test('detail page shows created_at', async ({ page, fixtureEventId }) => {
    await page.goto(`/dashboard/events/${fixtureEventId}`);
    await expect(page.locator('body')).toContainText('Created:');
  });

  test('detail page RSVPs table shows fixture attendee', async ({ page, fixtureEventId }) => {
    await page.goto(`/dashboard/events/${fixtureEventId}`);
    const rsvpRow = page.locator('tbody tr', { hasText: 'Test Attendee' });
    await expect(rsvpRow).toBeVisible();
    await expect(rsvpRow).toContainText('+12025550100');
    await expect(rsvpRow).toContainText('2');
  });

  test('detail page shows empty reminders state', async ({ page, fixtureEventId }) => {
    await page.goto(`/dashboard/events/${fixtureEventId}`);
    await expect(page.locator('body')).toContainText('No reminders configured.');
  });
});
