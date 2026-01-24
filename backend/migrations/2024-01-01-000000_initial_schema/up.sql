CREATE TABLE emails (
    id TEXT PRIMARY KEY NOT NULL,
    message_id TEXT,
    subject TEXT,
    date TIMESTAMP,
    envelope_from TEXT NOT NULL,
    size INTEGER NOT NULL,
    compressed_data BLOB NOT NULL,
    body_text TEXT,
    body_html TEXT,
    rendered_body_html TEXT,
    read BOOLEAN NOT NULL DEFAULT 0,
    has_attachments BOOLEAN NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE attachments (
    id TEXT PRIMARY KEY NOT NULL,
    email_id TEXT NOT NULL,
    filename TEXT,
    content_type TEXT,
    compressed_data BLOB NOT NULL,
    size INTEGER NOT NULL,
    content_id TEXT,
    disposition TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (email_id) REFERENCES emails (id) ON DELETE CASCADE
);

CREATE TABLE envelope_recipients (
    id TEXT PRIMARY KEY NOT NULL,
    email TEXT NOT NULL UNIQUE
);

CREATE TABLE email_envelope_recipients (
    email_id TEXT NOT NULL,
    envelope_recipient_id TEXT NOT NULL,
    PRIMARY KEY (email_id, envelope_recipient_id),
    FOREIGN KEY (email_id) REFERENCES emails (id) ON DELETE CASCADE,
    FOREIGN KEY (envelope_recipient_id) REFERENCES envelope_recipients (id) ON DELETE CASCADE
);

CREATE TABLE headers (
    id TEXT PRIMARY KEY NOT NULL,
    email_id TEXT NOT NULL,
    name TEXT NOT NULL,
    value TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (email_id) REFERENCES emails (id) ON DELETE CASCADE
);

-- Indexes for emails table
CREATE INDEX idx_emails_created_at ON emails(created_at DESC);
CREATE INDEX idx_emails_read ON emails(read);
CREATE INDEX idx_emails_message_id ON emails(message_id);
CREATE INDEX idx_emails_subject ON emails(subject);
CREATE INDEX idx_emails_envelope_from ON emails(envelope_from);

-- Indexes for attachments table
CREATE INDEX idx_attachments_email_id ON attachments(email_id);

-- Indexes for envelope_recipients table
CREATE INDEX idx_envelope_recipients_email ON envelope_recipients(email);

-- Indexes for email_envelope_recipients table
CREATE INDEX idx_email_envelope_recipients_email_id ON email_envelope_recipients(email_id);
CREATE INDEX idx_email_envelope_recipients_envelope_recipient_id ON email_envelope_recipients(envelope_recipient_id);

-- Indexes for headers table
CREATE INDEX idx_headers_email_id ON headers(email_id);
CREATE INDEX idx_headers_name ON headers(name);
