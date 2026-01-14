# MailFang

MailFang is the SMTP server and web interface for email testing that you have been waiting for!

## About

Use MailFang locally during development, deploy it in a sandbox, or use it in your staging environment instead of sending real emails.

### Features

* Easy deployment via docker or single file binary
* View rendered email content, inspect headers, attachments and raw email
* Check how your email reacts when users block remote content loading
* Preview emails in mobile, tablet, and desktop layouts
* Data is stored in sqlite database - file based or in-memory
* SMTP auth supports: `PLAIN`, `LOGIN`, `CRAM-MD5`, or no authentication
* Realtime email updates in the webui
* ... and much more

## Usage

### Docker

Use the [existing image](https://hub.docker.com/r/cars10/mailfang) from docker hub or from ghcr.io:

* `cars10/mailfang`
* `ghcr.io/cars10/mailfang`

```bash
docker run --name mailfang \
           -p 3000:3000 \
           -p 2525:2525 \
           -d cars10/mailfang
```

Default ports:

* `3000` - Web UI
* `2525` - SMTP Server


### Binary

1. Download the latest binary from the [releases page](https://github.com/cars10/mailfang/releases)
2. Set executable permissions: `chmod +x ./mailfang`
3. Run MailFang: `./mailfang`

You can view all available configuration options by running `./mailfang --help`.

## Configuration

You can configure MailFang via command-line arguments or environment variables. When running in docker, environment variables are recommended.

### Available Options

```bash
--smtp-host <SMTP_HOST>
    [env: SMTP_HOST=]
    [default: 0.0.0.0:2525]

--smtp-username <SMTP_USERNAME>
    SMTP authentication username
    [env: SMTP_USERNAME=]

--smtp-password <SMTP_PASSWORD>
    SMTP authentication password
    [env: SMTP_PASSWORD=]

--smtp-max-connections <SMTP_MAX_CONNECTIONS>
    Maximum number of concurrent SMTP connections
    [env: SMTP_MAX_CONNECTIONS=]
    [default: 4]

--web-host <WEB_HOST>
    [env: WEB_HOST=]
    [default: 0.0.0.0:3000]

--database-url <DATABASE_URL>
    SQLite database URL. Defaults to in-memory database.
    [env: DATABASE_URL=sqlite:///data/mailfang.db]
    [default: sqlite::memory:]
```

### Configuration Example

Persistent storage with smtp authentication:

```bash
mkdir -p mailfang/data
docker run --name mailfang \
           -p 3000:3000 \
           -p 2525:2525 \
           -e SMTP_USERNAME=user \
           -e SMTP_PASSWORD=pass \
           -e SMTP_MAX_CONNECTIONS=10 \
           -e DATABASE_URL=sqlite:///data/mailfang.db \
           -v ./mailfang/data:/data \
           cars10/mailfang
```

### Database Persistence

By default, MailFang uses an in-memory database. To persist emails between restarts configure a database url:

* Docker: Use `DATABASE_URL=sqlite:///data/mailfang.db` and mount a volume to `/data`
* Binary: Use `--database-url sqlite:///path/to/mailfang.db` or set `DATABASE_URL=sqlite:///path/to/mailfang.db`

## Development

### Prerequisites

* Docker and Docker Compose
* [Rust](https://rustup.rs/)
* Node.js

### Running

Run `make dev` to start the frontend and backend, access the frontend on `http://localhost:5173`.

## License

MIT
