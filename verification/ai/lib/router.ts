/**
 * Model Router for AI Verification
 *
 * Routes verification requests to configured AI models (Ollama or Claude).
 * Supports model configuration via TOML file and command-line overrides.
 */

import * as fs from 'node:fs';
import * as path from 'node:path';
import { Ollama } from 'ollama';
import Anthropic from '@anthropic-ai/sdk';
import * as toml from 'toml';
import type { CollectedArtifact } from './collector.js';
import type { Criterion } from './parser.js';

// Re-export types for convenience
export type { CollectedArtifact, Criterion };

/**
 * Supported AI providers
 */
export type Provider = 'ollama' | 'claude';

/**
 * Model configuration specifying provider and model name
 */
export interface ModelConfig {
  provider: Provider;
  model: string; // e.g., 'qwen3-vl:32b' or 'claude-sonnet-4-20250514'
}

/**
 * Analysis verdict returned by the model
 */
export interface Verdict {
  verdict: 'pass' | 'fail' | 'unclear';
  confidence: number; // 0-100
  evidence: string;
  suggestion: string | null;
}

/**
 * Configuration loaded from TOML file
 */
interface AIConfig {
  ai: {
    default_model: string;
    timeout?: number;
    confidence?: {
      high?: number;
      medium?: number;
    };
  };
}

/**
 * Error thrown when model is not available or unreachable
 */
export class ModelError extends Error {
  constructor(
    message: string,
    public readonly provider: Provider,
    public readonly model: string
  ) {
    super(message);
    this.name = 'ModelError';
  }
}

/**
 * The verification prompt template
 */
const VERIFICATION_PROMPT = `You are verifying if a UI artifact meets an acceptance criterion.

**Criterion:** {criterion}

**Additional context:** {annotation_hint}

Analyze the provided artifact and determine:
1. Does this artifact demonstrate the criterion is met?
2. What specific evidence supports your verdict?
3. How confident are you? (0-100%)

Respond in JSON:
{
  "verdict": "pass" | "fail" | "unclear",
  "confidence": <0-100>,
  "evidence": "<what you observed>",
  "suggestion": "<if fail, how to fix>" | null
}

IMPORTANT: Respond ONLY with valid JSON. No markdown code blocks, no additional text.`;

/**
 * Parse a model string into provider and model name.
 *
 * Format: "provider:model" where provider is "ollama" or "claude"
 *
 * Examples:
 *   "ollama:qwen3-vl:32b" -> { provider: 'ollama', model: 'qwen3-vl:32b' }
 *   "claude:claude-sonnet-4-20250514" -> { provider: 'claude', model: 'claude-sonnet-4-20250514' }
 *
 * @param modelString - The model string to parse
 * @returns Parsed model configuration
 * @throws Error if format is invalid
 */
export function parseModelString(modelString: string): ModelConfig {
  const colonIndex = modelString.indexOf(':');
  if (colonIndex === -1) {
    throw new Error(
      `Invalid model format: "${modelString}". Expected "provider:model" (e.g., "ollama:qwen3-vl:32b")`
    );
  }

  const provider = modelString.slice(0, colonIndex).toLowerCase();
  const model = modelString.slice(colonIndex + 1);

  if (provider !== 'ollama' && provider !== 'claude') {
    throw new Error(
      `Unknown provider: "${provider}". Supported providers: ollama, claude`
    );
  }

  if (!model) {
    throw new Error(`Model name cannot be empty`);
  }

  return { provider: provider as Provider, model };
}

/**
 * Load AI configuration from TOML file.
 *
 * @param configPath - Path to the config.toml file
 * @returns Parsed configuration
 * @throws Error if config file not found or invalid
 */
export function loadConfig(configPath: string): AIConfig {
  if (!fs.existsSync(configPath)) {
    throw new Error(`Config file not found: ${configPath}`);
  }

  const content = fs.readFileSync(configPath, 'utf-8');

  try {
    return toml.parse(content) as AIConfig;
  } catch (error) {
    throw new Error(
      `Failed to parse config file: ${error instanceof Error ? error.message : String(error)}`
    );
  }
}

/**
 * Get the default model configuration from config file or command-line override.
 *
 * @param configPath - Path to config.toml
 * @param modelOverride - Optional model string override from command line
 * @returns Model configuration
 */
export function getModelConfig(
  configPath: string,
  modelOverride?: string
): ModelConfig {
  if (modelOverride) {
    return parseModelString(modelOverride);
  }

  const config = loadConfig(configPath);
  return parseModelString(config.ai.default_model);
}

/**
 * Get the timeout from config file.
 *
 * @param configPath - Path to config.toml
 * @returns Timeout in milliseconds (default: 120000)
 */
export function getTimeout(configPath: string): number {
  try {
    const config = loadConfig(configPath);
    return (config.ai.timeout ?? 120) * 1000;
  } catch {
    return 120000;
  }
}

/**
 * Build the verification prompt from criterion and hint.
 *
 * @param criterion - The acceptance criterion
 * @param hint - Optional hint from annotation
 * @returns The formatted prompt
 */
function buildPrompt(criterion: Criterion, hint?: string): string {
  return VERIFICATION_PROMPT.replace('{criterion}', criterion.text).replace(
    '{annotation_hint}',
    hint ?? criterion.annotation?.hint ?? 'None'
  );
}

/**
 * Parse the model's JSON response into a Verdict.
 *
 * @param response - The raw response from the model
 * @returns Parsed verdict
 * @throws Error if response is not valid JSON or missing required fields
 */
export function parseVerdictResponse(response: string): Verdict {
  // Try to extract JSON from markdown code blocks if present
  let jsonStr = response.trim();

  // Remove markdown code blocks if present
  const jsonBlockMatch = jsonStr.match(/```(?:json)?\s*([\s\S]*?)```/);
  if (jsonBlockMatch) {
    jsonStr = jsonBlockMatch[1].trim();
  }

  let parsed: unknown;
  try {
    parsed = JSON.parse(jsonStr);
  } catch {
    throw new Error(`Failed to parse model response as JSON: ${response}`);
  }

  if (typeof parsed !== 'object' || parsed === null) {
    throw new Error(`Expected JSON object, got: ${typeof parsed}`);
  }

  const obj = parsed as Record<string, unknown>;

  // Validate verdict
  if (!['pass', 'fail', 'unclear'].includes(obj.verdict as string)) {
    throw new Error(
      `Invalid verdict: "${obj.verdict}". Expected: pass, fail, or unclear`
    );
  }

  // Validate confidence
  const confidence = Number(obj.confidence);
  if (isNaN(confidence) || confidence < 0 || confidence > 100) {
    throw new Error(
      `Invalid confidence: "${obj.confidence}". Expected number 0-100`
    );
  }

  // Validate evidence
  if (typeof obj.evidence !== 'string') {
    throw new Error(`Missing or invalid evidence field`);
  }

  // Validate suggestion (can be null or string)
  if (obj.suggestion !== null && typeof obj.suggestion !== 'string') {
    throw new Error(`Invalid suggestion field: expected string or null`);
  }

  return {
    verdict: obj.verdict as Verdict['verdict'],
    confidence: Math.round(confidence),
    evidence: obj.evidence,
    suggestion: obj.suggestion as string | null,
  };
}

/**
 * Analyze an artifact against a criterion using Ollama.
 *
 * @param artifact - The artifact to analyze
 * @param criterion - The criterion to verify
 * @param model - The Ollama model name
 * @param timeout - Timeout in milliseconds
 * @returns The analysis verdict
 */
async function analyzeWithOllama(
  artifact: CollectedArtifact,
  criterion: Criterion,
  model: string,
  timeout: number
): Promise<Verdict> {
  const ollama = new Ollama();

  // Check if Ollama is reachable
  try {
    await ollama.list();
  } catch {
    throw new ModelError(
      'Ollama not reachable. Run: ollama serve',
      'ollama',
      model
    );
  }

  // Check if model exists
  try {
    const models = await ollama.list();
    const modelExists = models.models.some(
      (m) => m.name === model || m.name.startsWith(model + ':')
    );
    if (!modelExists) {
      throw new ModelError(
        `Model not found. Run: ollama pull ${model}`,
        'ollama',
        model
      );
    }
  } catch (error) {
    if (error instanceof ModelError) throw error;
    throw new ModelError(
      `Failed to list Ollama models: ${error instanceof Error ? error.message : String(error)}`,
      'ollama',
      model
    );
  }

  const prompt = buildPrompt(criterion);

  // Create abort controller for timeout
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), timeout);

  try {
    // For images, send as base64
    const images =
      artifact.type === 'image' ? [artifact.data.toString('base64')] : [];

    const response = await ollama.generate({
      model,
      prompt,
      images,
      stream: false,
    });

    return parseVerdictResponse(response.response);
  } catch (error) {
    if (error instanceof Error && error.name === 'AbortError') {
      throw new ModelError('Analysis timed out', 'ollama', model);
    }
    throw error;
  } finally {
    clearTimeout(timeoutId);
  }
}

/**
 * Analyze an artifact against a criterion using Claude.
 *
 * @param artifact - The artifact to analyze
 * @param criterion - The criterion to verify
 * @param model - The Claude model name
 * @param timeout - Timeout in milliseconds
 * @returns The analysis verdict
 */
async function analyzeWithClaude(
  artifact: CollectedArtifact,
  criterion: Criterion,
  model: string,
  timeout: number
): Promise<Verdict> {
  // Claude requires ANTHROPIC_API_KEY environment variable
  if (!process.env.ANTHROPIC_API_KEY) {
    throw new ModelError(
      'ANTHROPIC_API_KEY environment variable not set',
      'claude',
      model
    );
  }

  const anthropic = new Anthropic();
  const prompt = buildPrompt(criterion);

  try {
    // Build content blocks
    const content: Anthropic.MessageCreateParams['messages'][0]['content'] = [];

    // For images, add as base64 content block
    if (artifact.type === 'image') {
      // Determine media type from file extension
      const ext = path.extname(artifact.path).toLowerCase();
      let mediaType: 'image/jpeg' | 'image/png' | 'image/gif' | 'image/webp' =
        'image/png';
      if (ext === '.jpg' || ext === '.jpeg') mediaType = 'image/jpeg';
      else if (ext === '.gif') mediaType = 'image/gif';
      else if (ext === '.webp') mediaType = 'image/webp';

      content.push({
        type: 'image',
        source: {
          type: 'base64',
          media_type: mediaType,
          data: artifact.data.toString('base64'),
        },
      });
    }

    // Add the text prompt
    content.push({
      type: 'text',
      text: prompt,
    });

    const response = await anthropic.messages.create({
      model,
      max_tokens: 1024,
      messages: [
        {
          role: 'user',
          content,
        },
      ],
    });

    // Extract text from response
    const textBlock = response.content.find((block) => block.type === 'text');
    if (!textBlock || textBlock.type !== 'text') {
      throw new Error('No text response from Claude');
    }

    return parseVerdictResponse(textBlock.text);
  } catch (error) {
    if (error instanceof ModelError) throw error;

    // Check for specific Anthropic errors
    if (error instanceof Anthropic.APIError) {
      if (error.status === 401) {
        throw new ModelError('Invalid ANTHROPIC_API_KEY', 'claude', model);
      }
      if (error.status === 404) {
        throw new ModelError(`Model not found: ${model}`, 'claude', model);
      }
      if (error.status === 408 || error.message.includes('timeout')) {
        throw new ModelError('Analysis timed out', 'claude', model);
      }
    }

    throw new ModelError(
      `Claude API error: ${error instanceof Error ? error.message : String(error)}`,
      'claude',
      model
    );
  }
}

/**
 * Analyze an artifact against a criterion using the configured model.
 *
 * This is the main entry point for the router.
 *
 * @param artifact - The artifact to analyze (image or video with data buffer)
 * @param criterion - The acceptance criterion to verify
 * @param modelOverride - Optional model string override (e.g., "ollama:llava:34b")
 * @param configPath - Path to config.toml (default: verification/config.toml)
 * @returns The analysis verdict with confidence and evidence
 */
export async function analyze(
  artifact: CollectedArtifact,
  criterion: Criterion,
  modelOverride?: string,
  configPath?: string
): Promise<Verdict> {
  // Resolve config path relative to project root
  const resolvedConfigPath =
    configPath ?? path.resolve(process.cwd(), 'verification/config.toml');

  const config = getModelConfig(resolvedConfigPath, modelOverride);
  const timeout = getTimeout(resolvedConfigPath);

  // Route to appropriate provider
  if (config.provider === 'ollama') {
    return analyzeWithOllama(artifact, criterion, config.model, timeout);
  } else {
    return analyzeWithClaude(artifact, criterion, config.model, timeout);
  }
}

/**
 * Get the default config path relative to the verification directory.
 *
 * @returns The default config path
 */
export function getDefaultConfigPath(): string {
  return path.resolve(process.cwd(), 'verification/config.toml');
}
