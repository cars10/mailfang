export interface Attachment {
  id: string
  filename: string | null
  content_type: string | null
  size: number
  content_id: string | null
  disposition: string | null
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
  recipients: string[] // SMTP envelope recipients (RCPT TO)
  size: number
  body_text: string | null
  body_html: string | null
  read: boolean
  attachments: Attachment[]
}

export interface EmailListRecord {
  id: string
  subject: string | null
  date: string | null
  created_at: string
  from: string
  recipients: string[]
  read: boolean
  has_attachments: boolean
}

export interface RecipientCount {
  recipient: string
  count: number
}

export interface EmailCounts {
  inbox: number
  unread: number
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
