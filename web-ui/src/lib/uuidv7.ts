// web-ui/src/lib/uuidv7.ts

/**
 * Extract the timestamp from a UUIDv7.
 *
 * UUIDv7 structure: first 48 bits = Unix timestamp in milliseconds
 * Format: xxxxxxxx-xxxx-7xxx-xxxx-xxxxxxxxxxxx
 *         ^^^^^^^^ ^^^^ = 12 hex chars = 48 bits of timestamp
 *
 * Also supports suffixed UUIDs like:
 *   xxxxxxxx-xxxx-7xxx-xxxx-xxxxxxxxxxxx-lightweight
 *   xxxxxxxx-xxxx-7xxx-xxxx-xxxxxxxxxxxx-checkpoint
 *
 * @param uuid - UUIDv7 string (with or without dashes, with optional suffix)
 * @returns Date object representing when the UUID was generated
 */
export function extractTimestampFromUuidv7(uuid: string): Date {
  // Strip any suffix after the UUID (e.g., "-lightweight", "-checkpoint")
  // UUIDs are 36 chars with dashes (32 hex + 4 dashes)
  const uuidPart = uuid.slice(0, 36);

  // Remove dashes if present
  const hex = uuidPart.replace(/-/g, '');

  // First 12 hex characters = 48 bits of timestamp (milliseconds)
  const timestampHex = hex.slice(0, 12);
  const timestampMs = parseInt(timestampHex, 16);

  return new Date(timestampMs);
}
