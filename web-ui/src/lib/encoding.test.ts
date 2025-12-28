import { describe, test, expect } from 'vitest';
import { encodeForTransport, decodeFromTransport } from './encoding';

describe('Terminal encoding utilities', () => {
  describe('encodeForTransport', () => {
    test('encodes ASCII text to base64', () => {
      const result = encodeForTransport('hello');
      expect(result).toBe('aGVsbG8=');
    });

    test('encodes Unicode text (emoji) to base64', () => {
      // This is the bug we're fixing - btoa() can't handle Unicode
      const result = encodeForTransport('hello \u{1F44B}');
      // UTF-8 bytes for "hello 👋" = 68 65 6c 6c 6f 20 f0 9f 91 8b
      expect(result).toBe('aGVsbG8g8J+Riw==');
    });

    test('encodes box-drawing characters to base64', () => {
      const result = encodeForTransport('───');
      // Each '─' (U+2500) is 3 UTF-8 bytes: E2 94 80
      expect(result).toBe('4pSA4pSA4pSA');
    });
  });

  describe('decodeFromTransport', () => {
    test('decodes base64 ASCII text', () => {
      const result = decodeFromTransport('aGVsbG8=');
      expect(result).toBe('hello');
    });

    test('decodes base64 Unicode text (emoji)', () => {
      const result = decodeFromTransport('aGVsbG8g8J+Riw==');
      expect(result).toBe('hello \u{1F44B}');
    });

    test('decodes base64 box-drawing characters', () => {
      const result = decodeFromTransport('4pSA4pSA4pSA');
      expect(result).toBe('───');
    });
  });

  describe('round-trip encoding', () => {
    test('ASCII survives round-trip', () => {
      const original = 'hello world';
      expect(decodeFromTransport(encodeForTransport(original))).toBe(original);
    });

    test('Unicode survives round-trip', () => {
      const original = 'Hello \u{1F30D}\u{1F44B} !';
      expect(decodeFromTransport(encodeForTransport(original))).toBe(original);
    });

    test('box-drawing characters survive round-trip', () => {
      const original = '┌───────┐\n│ test  │\n└───────┘';
      expect(decodeFromTransport(encodeForTransport(original))).toBe(original);
    });

    test('mixed content survives round-trip', () => {
      const original = '\u{250C}\u{2500}\u{2500} Hello \u{1F30D}\u{1F44B} \u{2500}\u{2500}\u{2510}';
      expect(decodeFromTransport(encodeForTransport(original))).toBe(original);
    });
  });
});
