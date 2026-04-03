import Database from 'better-sqlite3';
import path from 'path';

function getDbPath(): string {
  const url = process.env.DATABASE_URL;
  if (url) {
    const m = url.match(/^sqlite:\/\/(.+?)(\?.*)?$/);
    if (m) return path.resolve(__dirname, '../../', m[1]);
  }
  return path.resolve(__dirname, '../../are-you-going_development.sqlite');
}

const DB_PATH = getDbPath();

/** Most-recent magic link token for a given phone + event slug. */
export function getMagicLinkToken(phone: string, eventSlug: string): string {
  const db = new Database(DB_PATH, { readonly: true });
  try {
    const row = db.prepare<[string, string], { token: string }>(
      `SELECT ml.token FROM magic_links ml
       JOIN rsvps r  ON r.id = ml.rsvp_id
       JOIN events e ON e.id = r.event_id
       WHERE r.phone_number = ? AND e.slug = ?
       ORDER BY ml.created_at DESC LIMIT 1`
    ).get(phone, eventSlug);
    if (!row) throw new Error(`No magic link: phone=${phone} slug=${eventSlug}`);
    return row.token;
  } finally {
    db.close();
  }
}

/** Insert a phone number into sms_opt_outs (idempotent). */
export function insertOptOut(phone: string): void {
  const db = new Database(DB_PATH);
  try {
    db.prepare(`INSERT OR IGNORE INTO sms_opt_outs (phone_number, created_at, updated_at)
                VALUES (?, datetime('now'), datetime('now'))`).run(phone);
  } finally {
    db.close();
  }
}

/** Remove a phone number from sms_opt_outs. */
export function removeOptOut(phone: string): void {
  const db = new Database(DB_PATH);
  try {
    db.prepare(`DELETE FROM sms_opt_outs WHERE phone_number = ?`).run(phone);
  } finally {
    db.close();
  }
}

/** Event id + slug by name (most recently inserted). */
export function getEventByName(name: string): { id: number; slug: string } {
  const db = new Database(DB_PATH, { readonly: true });
  try {
    const row = db.prepare<[string], { id: number; slug: string }>(
      `SELECT id, slug FROM events WHERE name = ? ORDER BY id DESC LIMIT 1`
    ).get(name);
    if (!row) throw new Error(`Event not found: ${name}`);
    return row;
  } finally {
    db.close();
  }
}
