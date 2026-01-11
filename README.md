# MailFang

MailFang is the smtp server and webui for email testing that you have been waiting for!

## About

Use MailFang locally during development, deploy it in a sandbox or use it in your staging environment instead of sending real emails.

### Features

* Easy deployment via docker or self-contained single file binary
* View rendered email html or text content, inspect headers and raw email
* Check how your email reacts when users block remote content loading
* View email in mobile, tablet and desktop layout
* All data is saved in a local sqlite database or simply in-memory
* Connect to SMTP via `PLAIN`, `LOGIN`, `CRAM-MD5` auth or without authorization
* ... and much more 


## Usage

### Docker

Use the [existing image](https://hub.docker.com/r/cars10/mailfang):

```bash
docker run --name mailfang \
           -p 3000:3000 \
           -p 2525:2525 \
           -d cars10/mailfang
```

Default ports used by mailfang:

* `3000` - Webui
* `2525` - SMTP Server

### Binary

1. Download the latest binary from the [releases page](https://github.com/cars10/mailfang/releases)
2. Set executable permissions `sudo chmod +x ./mailfang`
3. Run mailfang `./mailfang`

## Configuration

Hint: You can run MailFang with `--help` to get the same information.

When running in docker the easiest way to configure MailFang to use the environment variables.

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

Full configuration example:

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

## License

MIT
