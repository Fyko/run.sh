# run.sh

> Execute sandboxed code on Discord

[Invite](https://discord.com/oauth2/authorize?client_id=1044442383288389692)

## Want a new language?

Create a [language request](https://github.com/1computer1/run.sh/issues/new?assignees=&labels=&template=feature_request.yml&title=).

## Development

1. Copy `.env.example` to `.envrc` and fill in the values.
2. Install [sqlx-cli] via `cargo`.
3. Run `sqlx db create` to create the database.
4. Run `sqlx migrate run` to create the database schema.
5. Run `cargo run` to start the bot. (optionally use `cargo make dev` to run with hot reloading)

## Development (tcp server)

If you're developing a new language and don't want to run the bot, you can run the tcp server with `cargo run --bin tcp` and connect with `nc localhost 8080`.

The .env file requirements are the same.

[sqlx-cli]: https://github.com/launchbadge/sqlx/tree/main/sqlx-cli
