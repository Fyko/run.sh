# run.sh

> Execute sandboxed code on Discord

[Invite](https://discord.com/oauth2/authorize?client_id=1044442383288389692)

## Want a new language?

Create a [language request](https://github.com/Fyko/run.sh/issues/new?assignees=&labels=&template=feature_request.yml&title=).

## Development

1. Copy `.env.example` to `.envrc` and fill in the values.
2. Install [sqlx-cli] via `cargo`.
3. Run `sqlx db create` to create the database.
4. Run `sqlx migrate run` to create the database schema.
5. Run `cargo run` to start the bot. (optionally use `cargo make dev` to run with hot reloading)

## Development (tcp server)

If you're developing a new language and don't want to run the bot, you can run the tcp server with `cargo run --bin tcp` and connect with `nc localhost 8080`.

The .env file requirements are the same except DISCORD_TOKEN can be any random string.

[sqlx-cli]: https://github.com/launchbadge/sqlx/tree/main/sqlx-cli

# Deployment (Linux only)

You have to [install](https://gvisor.dev/docs/user_guide/docker/) [gVisor](https://github.com/google/gvisor) as a runtime for docker to provide an additional isolation boundary between the containers and the host kernel.

```sh
# source https://gvisor.dev/docs/user_guide/install/#install-latest
(
  set -e
  ARCH=$(uname -m)
  URL=https://storage.googleapis.com/gvisor/releases/release/latest/${ARCH}
  wget ${URL}/runsc ${URL}/runsc.sha512 \
    ${URL}/containerd-shim-runsc-v1 ${URL}/containerd-shim-runsc-v1.sha512
  sha512sum -c runsc.sha512 \
    -c containerd-shim-runsc-v1.sha512
  rm -f *.sha512
  chmod a+rx runsc containerd-shim-runsc-v1
  sudo mv runsc containerd-shim-runsc-v1 /usr/local/bin
  sudo /usr/local/bin/runsc install
  sudo systemctl reload docker
)
```
