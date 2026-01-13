import { useState, useMemo } from 'react';
import { useModels, ModelInfo } from '../hooks/useModels';
import './Models.css';

export function ModelsPage() {
  const { models, providers, credentials, isLoading } = useModels();

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
        <div className="models-header">
          <h1>MODELS</h1>
        </div>
        <div className="models-content">
          <div className="models-loading">Loading models...</div>
        </div>
      </div>
    );
  }

  return (
    <div className="models-page">
      <div className="models-header">
        <h1>MODELS</h1>
      </div>

      <div className="models-content">
        {models.length === 0 ? (
          <div className="models-empty-state">
            <p>No models registered.</p>
            <p>Register a provider to see available models.</p>
          </div>
        ) : (
          <>
            <div className="models-filters">
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

            <div className="models-table-container">
              <table className="models-table">
                <thead>
                  <tr>
                    <th>Provider</th>
                    <th>Model</th>
                    <th>Context</th>
                    <th>Capabilities</th>
                  </tr>
                </thead>
                <tbody>
                  {filteredModels.map((model) => {
                    const credSource = credentialMap.get(model.provider);
                    return (
                      <tr
                        key={model.id}
                        className="models-row-clickable"
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
                        <td>{formatContext(model.context_window)}</td>
                        <td>{formatCapabilities(model.capabilities)}</td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>

            {selectedModel && (
              <ModelDetailsPanel
                model={selectedModel}
                credentialSource={credentialMap.get(selectedModel.provider)}
                onClose={() => setSelectedModel(null)}
              />
            )}
          </>
        )}
      </div>
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
      <div
        className="model-details-panel"
        role="dialog"
        aria-labelledby="model-details-title"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="model-details-header">
          <h2 id="model-details-title">{model.name}</h2>
          <button className="model-details-close" onClick={onClose} aria-label="Close">
            ×
          </button>
        </div>

        <div className="model-details-content">
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

            <dt>Capabilities</dt>
            <dd>{formatCapabilities(model.capabilities)}</dd>

            {model.pricing && (
              <>
                <dt>Pricing</dt>
                <dd>
                  ${model.pricing.input_per_million} per million input tokens
                  <br />
                  ${model.pricing.output_per_million} per million output tokens
                </dd>
              </>
            )}
          </dl>
        </div>
      </div>
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

function formatCapabilities(caps: { chat: boolean; vision: boolean; tools: boolean; embeddings: boolean; streaming: boolean }): string {
  const parts: string[] = [];
  if (caps.chat) parts.push('chat');
  if (caps.vision) parts.push('vision');
  if (caps.tools) parts.push('tools');
  if (caps.embeddings) parts.push('embeddings');
  return parts.length > 0 ? parts.join(', ') : '-';
}
