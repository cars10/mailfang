import { decodeMimeWords } from 'lettercoder'

export function decodeAddress(address: string): string {
  const parts = address.match(/^(.+?)\s*<(.+)>$/i)
  if (parts?.[1] && parts[2]) {
    const decodedName = decodeMimeWords(parts[1].trim())
    return `${decodedName} <${parts[2]}>`
  }
  return decodeMimeWords(address)
}

export function parseAddresses(addressString: string): string[] {
  const addresses: string[] = []
  let current = ''
  let inQuotes = false
  let inAngleBrackets = false
  let depth = 0

  for (const char of addressString) {
    if (char === '"' && !inAngleBrackets) {
      inQuotes = !inQuotes
      current += char
    } else if (char === '<' && !inQuotes) {
      inAngleBrackets = true
      depth++
      current += char
    } else if (char === '>' && !inQuotes) {
      inAngleBrackets = false
      depth--
      current += char
    } else if (char === ',' && !inQuotes && !inAngleBrackets && depth === 0) {
      const trimmed = current.trim()
      if (trimmed) addresses.push(trimmed)
      current = ''
    } else {
      current += char
    }
  }

  const trimmed = current.trim()
  if (trimmed) addresses.push(trimmed)

  return addresses
}

export function parseAndDecodeAddresses(headerValue: string): string[] {
  return parseAddresses(headerValue).map(decodeAddress)
}

export function parseAndDecodeHeaderValues(headerValues: string[]): string[] {
  return headerValues.flatMap(parseAndDecodeAddresses)
}
