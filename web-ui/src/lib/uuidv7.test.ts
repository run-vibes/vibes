// web-ui/src/lib/uuidv7.test.ts
import { describe, it, expect } from 'vitest';
import { extractTimestampFromUuidv7 } from './uuidv7';

describe('extractTimestampFromUuidv7', () => {
  it('extracts correct timestamp from valid UUIDv7', () => {
    // UUIDv7 with known timestamp: 1705321845123 (2024-01-15T12:30:45.123Z)
    // Hex: 018d0d1a5183 (12 chars = 48 bits)
    const uuid = '018d0d1a-5183-7000-8000-000000000000';
    const result = extractTimestampFromUuidv7(uuid);

    expect(result).toBeInstanceOf(Date);
    expect(result.getTime()).toBe(1705321845123);
  });

  it('handles UUIDs without dashes', () => {
    // Same timestamp as above, no dashes
    const uuid = '018d0d1a518370008000000000000000';
    const result = extractTimestampFromUuidv7(uuid);

    expect(result.getTime()).toBe(1705321845123);
  });

  it('extracts timestamp from recent event', () => {
    // 2025-01-02T00:00:00.000Z = 1735776000000
    // Hex: 0194244fd800
    const uuid = '0194244f-d800-7000-8000-000000000000';
    const result = extractTimestampFromUuidv7(uuid);

    // Use UTC methods to avoid timezone issues
    expect(result.getUTCFullYear()).toBe(2025);
    expect(result.getUTCMonth()).toBe(0); // January
    expect(result.getUTCDate()).toBe(2);
  });

  it('preserves millisecond precision', () => {
    // Two UUIDs 1ms apart: 1705321845123 and 1705321845124
    // Hex: 018d0d1a5183 and 018d0d1a5184
    const uuid1 = '018d0d1a-5183-7000-8000-000000000000';
    const uuid2 = '018d0d1a-5184-7000-8000-000000000000';

    const time1 = extractTimestampFromUuidv7(uuid1);
    const time2 = extractTimestampFromUuidv7(uuid2);

    expect(time2.getTime() - time1.getTime()).toBe(1);
  });

  it('returns valid date for epoch timestamp', () => {
    // Unix epoch: 1970-01-01T00:00:00.000Z = 0
    // Hex: 000000000000
    const epochUuid = '00000000-0000-7000-8000-000000000000';
    const epochResult = extractTimestampFromUuidv7(epochUuid);

    expect(epochResult.getTime()).toBe(0);
  });

  it('handles UUIDs with -lightweight suffix', () => {
    // Same timestamp as first test, with suffix
    const uuid = '018d0d1a-5183-7000-8000-000000000000-lightweight';
    const result = extractTimestampFromUuidv7(uuid);

    expect(result.getTime()).toBe(1705321845123);
  });

  it('handles UUIDs with -checkpoint suffix', () => {
    // Same timestamp as first test, with suffix
    const uuid = '018d0d1a-5183-7000-8000-000000000000-checkpoint';
    const result = extractTimestampFromUuidv7(uuid);

    expect(result.getTime()).toBe(1705321845123);
  });

  it('extracts same timestamp from suffixed and unsuffixed UUIDs', () => {
    const baseUuid = '018d0d1a-5183-7000-8000-000000000000';
    const lightweightUuid = `${baseUuid}-lightweight`;
    const checkpointUuid = `${baseUuid}-checkpoint`;

    const baseResult = extractTimestampFromUuidv7(baseUuid);
    const lightweightResult = extractTimestampFromUuidv7(lightweightUuid);
    const checkpointResult = extractTimestampFromUuidv7(checkpointUuid);

    expect(lightweightResult.getTime()).toBe(baseResult.getTime());
    expect(checkpointResult.getTime()).toBe(baseResult.getTime());
  });
});
