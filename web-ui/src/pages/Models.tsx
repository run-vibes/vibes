import { useState, useMemo } from 'react';
import { useModels } from '../hooks/useModels';
import './Models.css';

export function ModelsPage() {
  const { models, providers, credentials, isLoading } = useModels();

  // Build a map of provider -> credential source for quick lookup
  const credentialMap = new Map(credentials.map((c) => [c.provider, c.source]));
  const [providerFilter, setProviderFilter] = useState<string>('');

  const filteredModels = useMemo(() => {
    if (!providerFilter) return models;
    return models.filter((m) => m.provider === providerFilter);
  }, [models, providerFilter]);

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
                      <tr key={model.id}>
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
          </>
        )}
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
