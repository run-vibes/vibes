/**
 * Encode a string for WebSocket transport using base64.
 * Properly handles Unicode by encoding as UTF-8 first.
 */
export function encodeForTransport(data: string): string {
  const bytes = new TextEncoder().encode(data);
  const binaryString = String.fromCharCode(...bytes);
  return btoa(binaryString);
}

/**
 * Decode a base64 string received from WebSocket transport.
 * Properly handles UTF-8 decoding.
 */
export function decodeFromTransport(data: string): string {
  const binaryString = atob(data);
  const bytes = Uint8Array.from(binaryString, c => c.charCodeAt(0));
  return new TextDecoder('utf-8').decode(bytes);
}
