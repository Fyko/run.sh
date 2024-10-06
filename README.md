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
(
    set -e
    wget https://storage.googleapis.com/gvisor/releases/nightly/latest/runsc
    wget https://storage.googleapis.com/gvisor/releases/nightly/latest/runsc.sha512
    sha512sum -c runsc.sha512
    sudo mv runsc /usr/local/bin
    sudo chown root:root /usr/local/bin/runsc
    sudo chmod 0755 /usr/local/bin/runsc
)
```

`/etc/docker/daemon.json`:

```json
{
  "runtimes": {
    "runsc": {
      "path": "/usr/local/bin/runsc",
      "runtimeArgs": ["--network=none", "--overlay"]
    },
    "runsc-kvm": {
      "path": "/usr/local/bin/runsc",
      "runtimeArgs": ["--platform=kvm", "--network=none", "--overlay"]
    }
  }
}
```

You may have to create this file if it does not exist.
