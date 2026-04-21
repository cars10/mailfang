ALTER TABLE envelope_recipients
ADD COLUMN email_count INTEGER NOT NULL DEFAULT 0;

UPDATE envelope_recipients
SET email_count = (
    SELECT COUNT(*)
    FROM email_envelope_recipients
    WHERE email_envelope_recipients.envelope_recipient_id = envelope_recipients.id
);
