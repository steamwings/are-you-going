import { test, expect } from '@playwright/test';

// Use a timestamp suffix so repeated test runs don't hit duplicate-slug errors
const TS = Date.now();
const CRUD_SLUG = `crud-e2e-${TS}`;
const CRUD_EVENT_NAME = `CRUD Test Event ${TS}`;

test.describe('event CRUD', () => {
  test('new event form renders required fields', async ({ page }) => {
    await page.goto('/dashboard/events/new');
    await expect(page.locator('input[name="name"]')).toBeVisible();
    await expect(page.locator('textarea[name="description"]')).toBeVisible();
    await expect(page.locator('input[name="slug"]')).toBeVisible();
  });

  test('submit minimal form redirects to event detail', async ({ page }) => {
    await page.goto('/dashboard/events/new');
    await page.fill('input[name="name"]', CRUD_EVENT_NAME);
    await page.fill('textarea[name="description"]', 'Created by E2E');
    await page.fill('input[name="slug"]', CRUD_SLUG);
    await page.click('button[type="submit"]');
    await page.waitForURL('**/dashboard/events/**');
    await expect(page.locator('h1')).toContainText(CRUD_EVENT_NAME);
  });

  test('custom slug is preserved in public link', async ({ page }) => {
    const slugCheckSlug = `crud-slug-check-${TS}`;
    await page.goto('/dashboard/events/new');
    await page.fill('input[name="name"]', 'Slug Test Event');
    await page.fill('textarea[name="description"]', 'Testing slug');
    await page.fill('input[name="slug"]', slugCheckSlug);
    await page.click('button[type="submit"]');
    await page.waitForURL('**/dashboard/events/**');
    await expect(page.locator(`a[href*="/e/${slugCheckSlug}"]`)).toBeVisible();
  });

  test('duplicate slug shows error', async ({ page }) => {
    await page.goto('/dashboard/events/new');
    await page.fill('input[name="name"]', 'Duplicate Slug Event');
    await page.fill('textarea[name="description"]', 'Dup slug test');
    await page.fill('input[name="slug"]', CRUD_SLUG);
    await page.click('button[type="submit"]');
    await expect(page.locator('mark')).toContainText('already taken');
  });

  test('edit form pre-populates event name', async ({ page }) => {
    await page.goto('/dashboard');
    await page.locator('a', { hasText: CRUD_EVENT_NAME }).click();
    const url = page.url();
    const id = url.match(/\/dashboard\/events\/(\d+)/)?.[1];
    await page.goto(`/dashboard/events/${id}/edit`);
    await expect(page.locator('input[name="name"]')).toHaveValue(CRUD_EVENT_NAME);
  });

  test('updating event name persists on detail page', async ({ page }) => {
    await page.goto('/dashboard');
    await page.locator('a', { hasText: CRUD_EVENT_NAME }).click();
    const url = page.url();
    const id = url.match(/\/dashboard\/events\/(\d+)/)?.[1];
    await page.goto(`/dashboard/events/${id}/edit`);
    await page.fill('input[name="name"]', `${CRUD_EVENT_NAME} Updated`);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL(`/dashboard/events/${id}`);
    await expect(page.locator('h1')).toContainText(`${CRUD_EVENT_NAME} Updated`);
  });

  test('edit form has no slug input', async ({ page }) => {
    await page.goto('/dashboard');
    await page.locator('a', { hasText: `${CRUD_EVENT_NAME} Updated` }).click();
    const url = page.url();
    const id = url.match(/\/dashboard\/events\/(\d+)/)?.[1];
    await page.goto(`/dashboard/events/${id}/edit`);
    await expect(page.locator('input[name="slug"]')).toHaveCount(0);
  });

  test('unchecking show_name removes name input from RSVP form', async ({ page }) => {
    const noNameSlug = `crud-no-name-${TS}`;
    await page.goto('/dashboard/events/new');
    await page.fill('input[name="name"]', 'No Name Field Event');
    await page.fill('textarea[name="description"]', 'Testing field config');
    await page.fill('input[name="slug"]', noNameSlug);
    // show_name is checked by default — uncheck it
    await page.uncheck('input[name="show_name"]');
    await page.click('button[type="submit"]');
    await page.waitForURL('**/dashboard/events/**');

    await page.goto(`/e/${noNameSlug}`);
    await expect(page.locator('input[name="name"]')).toHaveCount(0);
  });

  test('deleting event removes it from the dashboard index', async ({ page }) => {
    // Create a fresh event specifically for deletion
    const delSlug = `crud-del-${TS}`;
    const delName = `Delete Test Event ${TS}`;
    await page.goto('/dashboard/events/new');
    await page.fill('input[name="name"]', delName);
    await page.fill('textarea[name="description"]', 'To be deleted');
    await page.fill('input[name="slug"]', delSlug);
    await page.click('button[type="submit"]');
    await page.waitForURL('**/dashboard/events/**');

    // Accept the confirmation dialog and click Delete
    page.once('dialog', dialog => dialog.accept());
    await page.click('button:has-text("Delete Event")');
    await page.waitForURL('**/dashboard');

    // Event should no longer appear in the index
    await expect(page.locator(`text=${delName}`)).toHaveCount(0);
  });
});
