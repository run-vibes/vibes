import { describe, test, expect, vi, beforeEach, afterEach } from 'vitest';
import * as fs from 'node:fs';
import {
  parseModelString,
  parseVerdictResponse,
  loadConfig,
  getModelConfig,
  ModelError,
  type ModelConfig,
  type Verdict,
} from './router.js';

// Mock fs module
vi.mock('node:fs', async () => {
  const actual = await vi.importActual<typeof import('node:fs')>('node:fs');
  return {
    ...actual,
    existsSync: vi.fn(),
    readFileSync: vi.fn(),
  };
});

describe('Model Router', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.resetAllMocks();
  });

  describe('parseModelString', () => {
    test('parses Ollama model string', () => {
      const result = parseModelString('ollama:qwen3-vl:32b');
      expect(result).toEqual({
        provider: 'ollama',
        model: 'qwen3-vl:32b',
      });
    });

    test('parses Claude model string', () => {
      const result = parseModelString('claude:claude-sonnet-4-20250514');
      expect(result).toEqual({
        provider: 'claude',
        model: 'claude-sonnet-4-20250514',
      });
    });

    test('handles uppercase provider', () => {
      const result = parseModelString('OLLAMA:llava:34b');
      expect(result).toEqual({
        provider: 'ollama',
        model: 'llava:34b',
      });
    });

    test('handles mixed case provider', () => {
      const result = parseModelString('Claude:claude-opus-4-20250514');
      expect(result).toEqual({
        provider: 'claude',
        model: 'claude-opus-4-20250514',
      });
    });

    test('throws for missing colon', () => {
      expect(() => parseModelString('ollama-qwen3-vl')).toThrow(
        'Invalid model format'
      );
    });

    test('throws for unknown provider', () => {
      expect(() => parseModelString('gemini:pro-vision')).toThrow(
        'Unknown provider: "gemini"'
      );
    });

    test('throws for empty model name', () => {
      expect(() => parseModelString('ollama:')).toThrow(
        'Model name cannot be empty'
      );
    });

    test('handles model name with multiple colons', () => {
      const result = parseModelString('ollama:qwen3-vl:32b:latest');
      expect(result).toEqual({
        provider: 'ollama',
        model: 'qwen3-vl:32b:latest',
      });
    });
  });

  describe('parseVerdictResponse', () => {
    test('parses valid pass verdict', () => {
      const json = JSON.stringify({
        verdict: 'pass',
        confidence: 92,
        evidence: 'The screenshot shows a sessions page',
        suggestion: null,
      });
      const result = parseVerdictResponse(json);
      expect(result).toEqual({
        verdict: 'pass',
        confidence: 92,
        evidence: 'The screenshot shows a sessions page',
        suggestion: null,
      });
    });

    test('parses valid fail verdict with suggestion', () => {
      const json = JSON.stringify({
        verdict: 'fail',
        confidence: 75,
        evidence: 'The button is missing',
        suggestion: 'Add a submit button to the form',
      });
      const result = parseVerdictResponse(json);
      expect(result).toEqual({
        verdict: 'fail',
        confidence: 75,
        evidence: 'The button is missing',
        suggestion: 'Add a submit button to the form',
      });
    });

    test('parses unclear verdict', () => {
      const json = JSON.stringify({
        verdict: 'unclear',
        confidence: 30,
        evidence: 'Cannot determine from artifact',
        suggestion: null,
      });
      const result = parseVerdictResponse(json);
      expect(result.verdict).toBe('unclear');
      expect(result.confidence).toBe(30);
    });

    test('rounds confidence to integer', () => {
      const json = JSON.stringify({
        verdict: 'pass',
        confidence: 87.5,
        evidence: 'Test evidence',
        suggestion: null,
      });
      const result = parseVerdictResponse(json);
      expect(result.confidence).toBe(88);
    });

    test('extracts JSON from markdown code block', () => {
      const response = `Here is my analysis:

\`\`\`json
{
  "verdict": "pass",
  "confidence": 85,
  "evidence": "Found the expected element",
  "suggestion": null
}
\`\`\``;
      const result = parseVerdictResponse(response);
      expect(result.verdict).toBe('pass');
      expect(result.confidence).toBe(85);
    });

    test('extracts JSON from code block without language specifier', () => {
      const response = `\`\`\`
{
  "verdict": "fail",
  "confidence": 60,
  "evidence": "Missing element",
  "suggestion": "Add the element"
}
\`\`\``;
      const result = parseVerdictResponse(response);
      expect(result.verdict).toBe('fail');
    });

    test('throws for invalid JSON', () => {
      expect(() => parseVerdictResponse('not json')).toThrow(
        'Failed to parse model response as JSON'
      );
    });

    test('throws for invalid verdict value', () => {
      const json = JSON.stringify({
        verdict: 'maybe',
        confidence: 50,
        evidence: 'test',
        suggestion: null,
      });
      expect(() => parseVerdictResponse(json)).toThrow('Invalid verdict');
    });

    test('throws for confidence out of range', () => {
      const json = JSON.stringify({
        verdict: 'pass',
        confidence: 150,
        evidence: 'test',
        suggestion: null,
      });
      expect(() => parseVerdictResponse(json)).toThrow('Invalid confidence');
    });

    test('throws for negative confidence', () => {
      const json = JSON.stringify({
        verdict: 'pass',
        confidence: -10,
        evidence: 'test',
        suggestion: null,
      });
      expect(() => parseVerdictResponse(json)).toThrow('Invalid confidence');
    });

    test('throws for missing evidence', () => {
      const json = JSON.stringify({
        verdict: 'pass',
        confidence: 80,
        suggestion: null,
      });
      expect(() => parseVerdictResponse(json)).toThrow(
        'Missing or invalid evidence'
      );
    });

    test('throws for invalid suggestion type', () => {
      const json = JSON.stringify({
        verdict: 'pass',
        confidence: 80,
        evidence: 'test',
        suggestion: 123,
      });
      expect(() => parseVerdictResponse(json)).toThrow(
        'Invalid suggestion field'
      );
    });
  });

  describe('loadConfig', () => {
    test('loads valid TOML config', () => {
      const configContent = `
[ai]
default_model = "ollama:qwen3-vl:32b"
timeout = 120

[ai.confidence]
high = 80
medium = 50
`;
      vi.mocked(fs.existsSync).mockReturnValue(true);
      vi.mocked(fs.readFileSync).mockReturnValue(configContent);

      const result = loadConfig('/path/to/config.toml');
      expect(result.ai.default_model).toBe('ollama:qwen3-vl:32b');
      expect(result.ai.timeout).toBe(120);
      expect(result.ai.confidence?.high).toBe(80);
      expect(result.ai.confidence?.medium).toBe(50);
    });

    test('throws for missing config file', () => {
      vi.mocked(fs.existsSync).mockReturnValue(false);

      expect(() => loadConfig('/nonexistent/config.toml')).toThrow(
        'Config file not found'
      );
    });

    test('throws for invalid TOML', () => {
      vi.mocked(fs.existsSync).mockReturnValue(true);
      vi.mocked(fs.readFileSync).mockReturnValue('invalid [ toml');

      expect(() => loadConfig('/path/to/config.toml')).toThrow(
        'Failed to parse config file'
      );
    });
  });

  describe('getModelConfig', () => {
    test('returns override when provided', () => {
      const result = getModelConfig('/any/path', 'claude:claude-opus-4-20250514');
      expect(result).toEqual({
        provider: 'claude',
        model: 'claude-opus-4-20250514',
      });
    });

    test('loads from config when no override', () => {
      const configContent = `
[ai]
default_model = "ollama:llava:34b"
`;
      vi.mocked(fs.existsSync).mockReturnValue(true);
      vi.mocked(fs.readFileSync).mockReturnValue(configContent);

      const result = getModelConfig('/path/to/config.toml');
      expect(result).toEqual({
        provider: 'ollama',
        model: 'llava:34b',
      });
    });
  });

  describe('ModelError', () => {
    test('includes provider and model in error', () => {
      const error = new ModelError(
        'Ollama not reachable',
        'ollama',
        'qwen3-vl:32b'
      );
      expect(error.message).toBe('Ollama not reachable');
      expect(error.provider).toBe('ollama');
      expect(error.model).toBe('qwen3-vl:32b');
      expect(error.name).toBe('ModelError');
    });

    test('can be caught and typed correctly', () => {
      const error = new ModelError('Test error', 'claude', 'claude-sonnet-4-20250514');
      expect(error instanceof ModelError).toBe(true);
      expect(error instanceof Error).toBe(true);
    });
  });
});

// Integration tests that would require actual API calls are skipped
describe.skip('Router Integration (requires running services)', () => {
  test('analyzes image with Ollama', async () => {
    // This test would require a running Ollama instance
  });

  test('analyzes image with Claude', async () => {
    // This test would require ANTHROPIC_API_KEY
  });
});
