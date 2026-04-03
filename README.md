# Are you going?

A self-hostable RSVP app with SMS reminders.

- 🎊 Create events and share links to invitees.
- 📋 They fill out a public form to RSVP to your event.
- 📫 Invitees can opt-in to SMS reminders.
- 🌐 Use the dashboard to view RSVPs and manage reminders.
- 🚂 Built with [Loco.rs](https://loco.rs/) ("Rust on Rails")

## Local Setup

```sh
cargo install --locked bacon  # helpful auto-restarting runner
bacon start                   # run dev server (uses .env.dev)
bacon test                    # run Rust tests
bacon e2e                     # run Playwright tests (requires `bacon start` to also be running)
```

## Deploy

A minimal docker-compose.yml is provided using [the public Docker hub image](https://hub.docker.com/r/steamwings/are-you-going).

Copy [.env.sample](./.env.sample) to `.env` wherever you're deploying and populate the values with real ones.

## SMS config

Currently only supports Vonage. Open to adding support for other APIs.

## License

AGPLv3 (copyleft). See [LICENSE](./LICENSE)

## FAQs

> What is the project goal?

To have a reasonably simple RSVP app I can host and use to get RSVPs from friends and family, and also send SMS reminders.

> Is this "production ready"?

I doubt it.
