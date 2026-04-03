export const BASE_URL = 'http://localhost:5150';

export const routes = {
  dashboard: {
    login:    '/dashboard/login',
    index:    '/dashboard',
    newEvent: '/dashboard/events/new',
    event:    (id: number) => `/dashboard/events/${id}`,
    editEvent:(id: number) => `/dashboard/events/${id}/edit`,
  },
  rsvp: {
    form:      (slug: string) => `/e/${slug}`,
    thanks:    (slug: string) => `/e/${slug}/thanks`,
    editPhone: (slug: string) => `/e/${slug}/edit`,
    editForm:  (slug: string, token: string) => `/e/${slug}/edit/${token}`,
  },
} as const;
