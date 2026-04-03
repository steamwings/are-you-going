import { test as base } from '@playwright/test';
import { getEventByName } from '../helpers/db';
import { FIXTURE_EVENT, FIXTURE_SLUG } from '../global.setup';

type F = { fixtureSlug: string; fixtureEventId: number };

export const test = base.extend<F>({
  fixtureSlug:    async ({}, use) => use(FIXTURE_SLUG),
  fixtureEventId: async ({}, use) => use(getEventByName(FIXTURE_EVENT).id),
});
export { expect } from '@playwright/test';
