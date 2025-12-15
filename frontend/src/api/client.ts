import type { EmailRecord, EmailListResponse, EmailCounts } from '@/types/email'

export class ApiClient {
  private baseUrl: string
  private defaultHeaders: HeadersInit

  constructor(baseUrl: string = '') {
    this.baseUrl = baseUrl
    this.defaultHeaders = {
      'Content-Type': 'application/json',
    }
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`
    const config: RequestInit = {
      ...options,
      headers: {
        ...this.defaultHeaders,
        ...options.headers,
      },
    }

    const response = await fetch(url, config)

    if (!response.ok) {
      throw new Error(`API request failed: ${response.statusText}`)
    }

    if (
      response.status === 204 ||
      response.headers.get('content-length') === '0'
    ) {
      return undefined as T
    }

    return response.json()
  }

  async inbox(page: number = 1, search?: string): Promise<EmailListResponse> {
    const searchParam = search ? `&search=${encodeURIComponent(search)}` : ''
    return this.request<EmailListResponse>(
      `/api/emails?page=${page}${searchParam}`
    )
  }

  async getSidebar(): Promise<EmailCounts> {
    return this.request<EmailCounts>('/api/emails/sidebar')
  }

  async getEmail(id: string): Promise<EmailRecord> {
    return this.request<EmailRecord>(`/api/emails/${id}`)
  }

  async deleteEmail(id: string): Promise<void> {
    return this.request<void>(`/api/emails/${id}`, {
      method: 'DELETE',
    })
  }

  async deleteAll(): Promise<void> {
    return this.request<void>('/api/emails', {
      method: 'DELETE',
    })
  }

  getAttachmentUrl(id: string): string {
    return `${this.baseUrl}/api/attachments/${id}`
  }

  async getRawEmail(id: string): Promise<string> {
    const url = `${this.baseUrl}/api/emails/${id}/raw`
    const response = await fetch(url)

    if (!response.ok) {
      throw new Error(`API request failed: ${response.statusText}`)
    }

    return response.text()
  }
}

export const apiClient = new ApiClient()
