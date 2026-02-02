# MailFang

MailFang is the email testing tool that you've been waiting for. It provides a local smtp server and a modern webui to view your emails.

**[Live Demo](https://demo.mailfang.com)**

## About

Use MailFang locally during development, deploy it in a sandbox, or use it in your staging environment instead of sending emails to an external service. Regain control of your data and eliminate unnecessary costs.

### Features

* Easy deployment via docker or single file binary
* View rendered email content, inspect headers, attachments and raw emails
* Preview emails in different viewports and block remote content loading
* Realtime updates in the webui
* ... and much more

## Quickstart

```bash
docker run --name mailfang \
           -p 3000:3000 \
           -p 2525:2525 \
           -d cars10/mailfang
```

Open the webui on [0.0.0.0:3000](http://0.0.0.0:3000) and send emails to `0.0.0.0:2525`.

## Usage

By default mailfang will use the following ports:

* Webui: `3000` 
* SMTP: `2525` 

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

The services will be reachable via:

* Webui: [0.0.0.0:3000](http://0.0.0.0:3000)
* SMTP: `0.0.0.0:2525`

### Binary

1. Download the latest binary from the [releases page](https://github.com/cars10/mailfang/releases)
2. Set executable permissions: `chmod +x ./mailfang`
3. Run MailFang: `./mailfang`

You can view all available configuration options by running `./mailfang --help`.

## Sending emails to MailFang

To send emails to Mailfang simply configure your email service to use the respective address and port where MailFang is running. Assuming you run MailFang in docker with default settings:

### Rails ActionMailer

```rails
config.action_mailer.smtp_settings = {
  address:         "0.0.0.0",
  port:            2525,
  domain:          "example.com",
  authentication:  "plain"
}
```

### Swaks

You can do a simple test on the command line via swaks:

```bash
swaks --to recipient@example.com \
      --from sender@example.com \
      --server 0.0.0.0:2525 \
      --header "From: sender@example.com" \
      --header "To: recipient@example.com" \
      --header "Subject: Test Email" \
      --body "This is a plain text message."
```

## Configuration

You can configure MailFang via command-line arguments or environment variables. When running in docker, environment variables are recommended.

### Available Options

All configuration options are optional. The smtp server will accept all connection if no credentials are configured.

| Option | Environment Variable | Description | Binary Default | Docker Default |
|--------|---------------------|-------------|----------------|----------------|
| `--smtp-host` | `SMTP_HOST` | SMTP server listen address | `127.0.0.1:2525` | `0.0.0.0:2525` |
| `--smtp-username` | `SMTP_USERNAME` | SMTP authentication username | _none_ | _none_ |
| `--smtp-password` | `SMTP_PASSWORD` | SMTP authentication password | _none_ | _none_ |
| `--smtp-max-connections` | `SMTP_MAX_CONNECTIONS` | Maximum number of concurrent SMTP connections | `4` | `4` |
| `--web-host` | `WEB_HOST` | Web server listen address | `127.0.0.1:3000` | `0.0.0.0:3000` |
| `--database-url` | `DATABASE_URL` | SQLite database URL | `sqlite://./mailfang.db` | `sqlite:///data/mailfang.db` |

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

MailFang saves emails in a local sqlite database. To persist the data:

When running via docker use `DATABASE_URL=sqlite:///data/mailfang.db` and mount a volume to `/data`.

The binary defaults to `./mailfang.db`, change it by using `--database-url sqlite:///path/to/mailfang.db` or via environment variables `DATABASE_URL=sqlite:///path/to/mailfang.db`

## SMTP Server

The SMTP server mostly implements [RFC 5321 - Simple Mail Transfer Protocol](https://datatracker.ietf.org/doc/html/rfc5321).

It supports the following authorization methods:

* no auth
* `PLAIN`
* `LOGIN`
* `CRAM-MD5`

By default it only accepts a maximum of `4` emails at the same time. This is configurable via `--smtp-max-connections 12` or `SMTP_MAX_CONNECTIONS=12`.

## Development

### Prerequisites

* Docker and Docker Compose
* [Rust](https://rustup.rs/)
* Node.js

### Running

Run `make dev` to start the frontend and backend, access the frontend on `http://localhost:5173`.

## License

MIT
