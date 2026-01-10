---
id: 39-models-registry-auth
title: Models Registry & Auth
status: planned
epics: [models]
---

# Models Registry & Auth

## Overview

First milestone of the Models epic. Establishes the foundation for model management: registry for discovering available models, credential management for API keys, and the provider trait abstraction.

## Goals

- Model catalog with capability discovery
- Secure credential management (system keyring + env fallback)
- Provider trait defining the inference interface
- CLI commands for listing models and managing credentials

## Key Deliverables

- `vibes-models` crate skeleton
- `ModelRegistry` with model discovery
- `CredentialStore` for API key management
- `ModelProvider` trait definition
- `vibes models list` and `vibes models auth` commands

## Epics

- [models](../../epics/models)

## Design

See [../../epics/models/README.md](../../epics/models/README.md) for architecture.
