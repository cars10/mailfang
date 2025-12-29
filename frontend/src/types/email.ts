export interface EmailAttachment {
  id: string
  filename: string | null
  mime_type: string
  size: number
  content_id: string | null
  headers: Record<string, string[]> | null
  created_at: string
}

export interface EmailRecord {
  id: string
  message_id: string | null
  subject: string | null
  date: string | null
  headers: Record<string, string[]> | null
  created_at: string
  from: string // SMTP envelope sender (MAIL FROM)
  to: string[] // SMTP envelope recipients (RCPT TO)
  recipients: string[] // Same as `to` - all SMTP envelope recipients
  size: number
  body_text: string
  body_html: string
  read: boolean
  attachments: EmailAttachment[]
}

export interface EmailListRecord {
  id: string
  subject: string | null
  date: string | null
  created_at: string
  from: string
  to: string[]
  read: boolean
  has_attachments: boolean
}

export interface RecipientCount {
  recipient: string
  count: number
}

export interface EmailCounts {
  inbox: number
  recipients: RecipientCount[]
}

export interface PaginationInfo {
  page: number
  per_page: number
  total_pages: number
}

export interface EmailListResponse {
  emails: EmailListRecord[]
  counts: EmailCounts
  pagination: PaginationInfo
}
