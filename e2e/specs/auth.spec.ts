import { test, expect } from '@playwright/test';

test.describe('auth', () => {
  test('login page renders', async ({ page }) => {
    await page.goto('/dashboard/login');
    await expect(page.locator('input[name="password"]')).toBeVisible();
  });

  test('wrong password stays on login with error', async ({ page }) => {
    await page.goto('/dashboard/login');
    await page.fill('input[name="password"]', 'wrongpassword');
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL(/\/dashboard\/login/);
    await expect(page.locator('mark')).toBeVisible();
  });

  test('correct password redirects to dashboard', async ({ page }) => {
    const password = process.env.DASHBOARD_PASSWORD ?? 'testpassword';
    await page.goto('/dashboard/login');
    await page.fill('input[name="password"]', password);
    await page.click('button[type="submit"]');
    await page.waitForURL('**/dashboard');
    await expect(page).toHaveURL(/\/dashboard$/);
  });

  test('unauthenticated /dashboard redirects to login', async ({ page }) => {
    await page.goto('/dashboard');
    await expect(page).toHaveURL(/\/dashboard\/login/);
  });

  test('unauthenticated /dashboard/events/new redirects to login', async ({ page }) => {
    await page.goto('/dashboard/events/new');
    await expect(page).toHaveURL(/\/dashboard\/login/);
  });

  test('manual cookie injection grants dashboard access', async ({ page }) => {
    await page.context().addCookies([
      {
        name: 'dashboard_auth',
        value: 'authenticated',
        domain: 'localhost',
        path: '/',
      },
    ]);
    await page.goto('/dashboard');
    await expect(page).not.toHaveURL(/\/dashboard\/login/);
    await expect(page).toHaveURL(/\/dashboard/);
  });
});
