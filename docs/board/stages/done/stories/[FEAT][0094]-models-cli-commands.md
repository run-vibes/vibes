---
id: FEAT0094
title: vibes models CLI commands
type: feat
status: done
priority: high
epics: [models]
depends: [FEAT0092, FEAT0093]
estimate: 3h
created: 2026-01-13
milestone: 37-models-registry-auth
---

# vibes models CLI commands

## Summary

Add CLI commands for listing models and managing API credentials.

## Commands

### vibes models list

List available models with their capabilities.

```
vibes models list                    # List all models
vibes models list --provider openai  # Filter by provider
vibes models list --capability vision # Filter by capability
```

Output:
```
Provider    Model                 Context   Capabilities
anthropic   claude-sonnet-4       200K      chat, vision, tools
anthropic   claude-haiku-3        200K      chat, tools
openai      gpt-4o                128K      chat, vision, tools
openai      text-embedding-3-large 8K       embeddings
```

### vibes models info

Show detailed model information.

```
vibes models info claude-sonnet-4
```

### vibes models auth

Configure API credentials for a provider.

```
vibes models auth anthropic          # Interactive prompt for key
vibes models auth anthropic --delete # Remove stored key
vibes models auth --list             # Show configured providers
```

## Implementation

- Add `models` subcommand to vibes-cli
- Use `dialoguer` for interactive input
- Hide API key input with password masking
- Table output with `comfy-table`

## Acceptance Criteria

- [x] `vibes models list` with filtering options
- [x] `vibes models info <model>` shows details
- [x] `vibes models auth <provider>` stores in keyring
- [x] `vibes models auth --list` shows configured providers
- [x] `vibes models auth --delete` removes credentials
- [x] Unit tests for CLI commands
