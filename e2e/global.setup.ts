import { chromium } from '@playwright/test';
import path from 'path';
import fs from 'fs';
import { STORAGE_STATE } from '../playwright.config';

export const FIXTURE_SLUG  = 'e2e-fixture';
export const FIXTURE_EVENT = 'E2E Fixture Event';
export const FIXTURE_PHONE = '+12025550100';

export default async function globalSetup() {
  fs.mkdirSync(path.dirname(STORAGE_STATE), { recursive: true });

  const browser = await chromium.launch();
  const ctx     = await browser.newContext();
  const page    = await ctx.newPage();

  const password = process.env.DASHBOARD_PASSWORD ?? 'testpassword';

  // 1. Log in
  await page.goto('http://localhost:5150/dashboard/login');
  await page.fill('input[name="password"]', password);
  await page.click('button[type="submit"]');
  await page.waitForURL('**/dashboard');

  // 2. Save auth state
  await ctx.storageState({ path: STORAGE_STATE });

  // 3. Create shared fixture event (idempotent — skip if slug already exists)
  await page.goto('http://localhost:5150/dashboard/events/new');
  await page.fill('input[name="name"]', FIXTURE_EVENT);
  await page.fill('textarea[name="description"]', 'E2E fixture event');
  await page.fill('input[name="slug"]', FIXTURE_SLUG);
  await page.check('input[name="show_allergies"]');
  await page.click('button[type="submit"]');

  // If slug was already taken, we stay on the form with an error — that's fine
  const afterSubmitUrl = page.url();
  if (!afterSubmitUrl.includes('/dashboard/events/')) {
    // Event already exists; find its URL via the index
    await page.goto('http://localhost:5150/dashboard');
  }

  // 4. Create shared fixture RSVP (idempotent — skip if phone already exists)
  await page.goto(`http://localhost:5150/e/${FIXTURE_SLUG}`);
  await page.fill('input[name="name"]', 'Test Attendee');
  await page.fill('input[name="phone_number"]', '2025550100');
  await page.fill('input[name="party_size"]', '2');
  await page.click('button[type="submit"]');
  // Either lands on /thanks (new RSVP) or stays on the form with "already exists" error — both are fine
  await page.waitForLoadState('networkidle');

  await browser.close();
}
