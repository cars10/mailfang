CREATE TABLE emails (
    id TEXT PRIMARY KEY NOT NULL,
    message_id TEXT,
    subject TEXT,
    date TIMESTAMP,
    headers TEXT,
    "from" TEXT NOT NULL,
    size INTEGER NOT NULL,
    raw_data TEXT NOT NULL,
    body_text TEXT,
    body_html TEXT,
    rendered_body_html TEXT,
    read BOOLEAN NOT NULL DEFAULT 0,
    has_attachments BOOLEAN NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE email_attachments (
    id TEXT PRIMARY KEY NOT NULL,
    email_id TEXT NOT NULL,
    filename TEXT,
    mime_type TEXT NOT NULL,
    data BLOB NOT NULL,
    size INTEGER NOT NULL,
    content_id TEXT,
    headers TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (email_id) REFERENCES emails (id) ON DELETE CASCADE
);

CREATE TABLE recipients (
    id TEXT PRIMARY KEY NOT NULL,
    email TEXT NOT NULL UNIQUE
);

CREATE TABLE email_recipients (
    email_id TEXT NOT NULL,
    recipient_id TEXT NOT NULL,
    PRIMARY KEY (email_id, recipient_id),
    FOREIGN KEY (email_id) REFERENCES emails (id) ON DELETE CASCADE,
    FOREIGN KEY (recipient_id) REFERENCES recipients (id) ON DELETE CASCADE
);
