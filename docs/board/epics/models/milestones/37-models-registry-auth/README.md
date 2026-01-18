---
id: 37-models-registry-auth
title: Model Management
status: done
epics: [models]
---

# Model Management

## Overview

First milestone of the Models epic. Establishes the foundation for model management: registry for discovering available models, credential management for API keys, and the provider trait abstraction.

## Goals

- Model catalog with capability discovery
- Secure credential management (system keyring + env fallback)
- Provider trait defining the inference interface
- CLI commands for listing models and managing credentials
- Web UI for browsing models and managing credentials

## Stories

| ID | Title | Status |
|----|-------|--------|
| FEAT0090 | [vibes-models crate skeleton](../../stages/backlog/stories/feat-0090-vibes-models-crate-skeleton.md) | pending |
| FEAT0091 | [ModelProvider trait definition](../../stages/backlog/stories/feat-0091-model-provider-trait.md) | pending |
| FEAT0092 | [ModelRegistry for model discovery](../../stages/backlog/stories/feat-0092-model-registry.md) | pending |
| FEAT0093 | [CredentialStore for API key management](../../stages/backlog/stories/feat-0093-credential-store.md) | pending |
| FEAT0094 | [vibes models CLI commands](../../stages/backlog/stories/feat-0094-models-cli-commands.md) | pending |
| FEAT0095 | [Models view in web UI](../../stages/backlog/stories/feat-0095-models-web-ui.md) | pending |

## Key Deliverables

- `vibes-models` crate skeleton
- `ModelRegistry` with model discovery
- `CredentialStore` for API key management
- `ModelProvider` trait definition
- `vibes models list` and `vibes models auth` commands
- Models page in web UI with credentials management

## Epics

- [models](../../epics/models)

## Design

See [../../epics/models/README.md](../../epics/models/README.md) for architecture.
