import { useState, useMemo } from 'react';
import { PageHeader, Card, EmptyState } from '@vibes/design-system';
import { useModels, useWebSocket } from '../hooks';
import type { ModelInfo } from '../lib/types';
import './Models.css';

export function ModelsPage() {
  const { send, addMessageHandler, isConnected } = useWebSocket();
  const { models, providers, credentials, isLoading } = useModels({
    send,
    addMessageHandler,
    isConnected,
    autoRefresh: true,
  });

  // Build a map of provider -> credential source for quick lookup
  const credentialMap = new Map(credentials.map((c) => [c.provider, c.source]));
  const [providerFilter, setProviderFilter] = useState<string>('');
  const [capabilityFilter, setCapabilityFilter] = useState<string>('');
  const [selectedModel, setSelectedModel] = useState<ModelInfo | null>(null);

  const filteredModels = useMemo(() => {
    return models.filter((m) => {
      if (providerFilter && m.provider !== providerFilter) return false;
      if (capabilityFilter && !m.capabilities[capabilityFilter as keyof typeof m.capabilities]) return false;
      return true;
    });
  }, [models, providerFilter, capabilityFilter]);

  if (isLoading) {
    return (
      <div className="models-page">
        <PageHeader title="MODELS" />
        <p className="models-loading">Loading models...</p>
      </div>
    );
  }

  return (
    <div className="models-page">
      <PageHeader title="MODELS" />

      {models.length === 0 ? (
        <Card variant="crt">
          <EmptyState
            icon="⚡"
            message="No models registered"
            hint="Register a provider to see available models."
          />
        </Card>
      ) : (
        <div className="models-content">
          <div className="models-filters">
            <div className="models-filter">
              <label htmlFor="provider-filter">Provider</label>
              <select
                id="provider-filter"
                value={providerFilter}
                onChange={(e) => setProviderFilter(e.target.value)}
              >
                <option value="">All providers</option>
                {providers.map((p) => (
                  <option key={p} value={p}>
                    {p}
                  </option>
                ))}
              </select>
            </div>

            <div className="models-filter">
              <label htmlFor="capability-filter">Capability</label>
              <select
                id="capability-filter"
                value={capabilityFilter}
                onChange={(e) => setCapabilityFilter(e.target.value)}
              >
                <option value="">All capabilities</option>
                <option value="chat">Chat</option>
                <option value="vision">Vision</option>
                <option value="tools">Tools</option>
                <option value="embeddings">Embeddings</option>
              </select>
            </div>
          </div>

          <Card variant="crt" noPadding>
            <div className="models-table-container">
              <table className="models-table">
                <thead>
                  <tr>
                    <th>Provider</th>
                    <th>Model</th>
                    <th>Size</th>
                    <th>Context</th>
                    <th>Modified</th>
                    <th>Capabilities</th>
                  </tr>
                </thead>
                <tbody>
                  {filteredModels.map((model) => {
                    const credSource = credentialMap.get(model.provider);
                    return (
                      <tr
                        key={model.id}
                        className="models-row"
                        onClick={() => setSelectedModel(model)}
                      >
                        <td className="provider-cell">
                          {model.provider}
                          {credSource && (
                            <span
                              className="auth-indicator"
                              title={`Authenticated via ${credSource}`}
                            >
                              ✓
                            </span>
                          )}
                        </td>
                        <td>{model.name}</td>
                        <td className="size-cell">{formatSize(model.size_bytes)}</td>
                        <td className="context-cell">{formatContext(model.context_window)}</td>
                        <td className="modified-cell">{formatModified(model.modified_at)}</td>
                        <td className="capabilities-cell">{formatCapabilities(model.capabilities)}</td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </Card>

          {selectedModel && (
            <ModelDetailsPanel
              model={selectedModel}
              credentialSource={credentialMap.get(selectedModel.provider)}
              onClose={() => setSelectedModel(null)}
            />
          )}
        </div>
      )}
    </div>
  );
}

interface ModelDetailsPanelProps {
  model: ModelInfo;
  credentialSource?: string;
  onClose: () => void;
}

function ModelDetailsPanel({ model, credentialSource, onClose }: ModelDetailsPanelProps) {
  return (
    <div className="model-details-overlay" onClick={onClose}>
      <Card
        variant="crt"
        title={model.name}
        className="model-details-panel"
        actions={
          <button className="model-details-close" onClick={onClose} aria-label="Close">
            ×
          </button>
        }
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-labelledby="model-details-title"
      >
        <dl className="model-details-list">
          <dt>Provider</dt>
          <dd>
            {model.provider}
            {credentialSource && (
              <span className="auth-indicator" title={`Authenticated via ${credentialSource}`}>
                ✓
              </span>
            )}
          </dd>

          <dt>Context Window</dt>
          <dd>{formatContext(model.context_window)} tokens</dd>

          {model.size_bytes && (
            <>
              <dt>Size</dt>
              <dd>{formatSize(model.size_bytes)}</dd>
            </>
          )}

          {model.modified_at && (
            <>
              <dt>Modified</dt>
              <dd>{formatModified(model.modified_at)}</dd>
            </>
          )}

          <dt>Capabilities</dt>
          <dd>{formatCapabilities(model.capabilities)}</dd>

          {model.pricing && (
            <>
              <dt>Pricing</dt>
              <dd>
                ${model.pricing.input_per_million}/M input
                <br />
                ${model.pricing.output_per_million}/M output
              </dd>
            </>
          )}
        </dl>
      </Card>
    </div>
  );
}

function formatContext(tokens: number): string {
  if (tokens >= 1_000_000) {
    return `${tokens / 1_000_000}M`;
  } else if (tokens >= 1_000) {
    return `${tokens / 1_000}K`;
  }
  return String(tokens);
}

function formatSize(bytes?: number): string {
  if (bytes === undefined || bytes === null) {
    return '-';
  }
  const gb = bytes / (1024 * 1024 * 1024);
  if (gb >= 1) {
    return `${gb.toFixed(1)} GB`;
  }
  const mb = bytes / (1024 * 1024);
  return `${mb.toFixed(0)} MB`;
}

function formatModified(isoDate?: string): string {
  if (!isoDate) {
    return '-';
  }
  const date = new Date(isoDate);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

  if (diffDays === 0) {
    return 'Today';
  } else if (diffDays === 1) {
    return 'Yesterday';
  } else if (diffDays < 7) {
    return `${diffDays} days ago`;
  } else if (diffDays < 30) {
    const weeks = Math.floor(diffDays / 7);
    return `${weeks} week${weeks > 1 ? 's' : ''} ago`;
  } else {
    return date.toLocaleDateString();
  }
}

function formatCapabilities(caps: { chat: boolean; vision: boolean; tools: boolean; embeddings: boolean; streaming: boolean }): string {
  const parts: string[] = [];
  if (caps.chat) parts.push('chat');
  if (caps.vision) parts.push('vision');
  if (caps.tools) parts.push('tools');
  if (caps.embeddings) parts.push('embeddings');
  return parts.length > 0 ? parts.join(', ') : '-';
}
