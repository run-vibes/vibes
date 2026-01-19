---
id: FEAT0095
title: Models view in web UI
type: feat
status: done
priority: medium
scope: models
depends: [FEAT0092, FEAT0093]
estimate: 4h
created: 2026-01-13
---

# Models view in web UI

## Summary

Add a models management page to the web UI for browsing available models and managing API credentials.

## Features

### Models List View

Display available models with filtering and search:

- Table showing provider, model name, context window, capabilities
- Filter by provider (dropdown)
- Filter by capability (chat, vision, tools, embeddings)
- Search by model name
- Click model row to show details panel

### Model Details Panel

Slide-out or modal showing:

- Full model information (pricing, limits, description)
- Provider status (authenticated or not)
- Quick action to configure credentials if missing

### Credentials Management

Settings section for API key management:

- List configured providers with status indicator
- Add/update credentials (masked input)
- Remove credentials with confirmation
- Show environment variable fallback status

## Implementation

- New route: `/models`
- Components:
  - `ModelsPage` - main container
  - `ModelsTable` - sortable/filterable table
  - `ModelDetails` - detail panel
  - `CredentialsSettings` - auth management
- API endpoints (via existing WebSocket):
  - `models.list` - list models with filters
  - `models.info` - get model details
  - `credentials.list` - list configured providers
  - `credentials.set` - store credential
  - `credentials.delete` - remove credential

## Acceptance Criteria

- [ ] Models list page with table view
- [ ] Filter by provider and capability
- [ ] Model details panel
- [ ] Credentials management in settings
- [ ] Visual indicator for authenticated providers
- [ ] Responsive layout for mobile
