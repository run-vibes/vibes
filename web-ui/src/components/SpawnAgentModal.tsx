import { useState } from 'react';
import { Button } from '@vibes/design-system';
import type { AgentType } from '../lib/types';
import './SpawnAgentModal.css';

interface SpawnAgentModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSpawn: (type: AgentType, name?: string, task?: string) => void;
  isSpawning: boolean;
}

const agentTypes: { value: AgentType; label: string; description: string }[] = [
  {
    value: 'AdHoc',
    label: 'Ad-hoc',
    description: 'User-triggered, interactive agent for one-off tasks',
  },
  {
    value: 'Background',
    label: 'Background',
    description: 'Long-running, autonomous agent for scheduled tasks',
  },
  {
    value: 'Subagent',
    label: 'Subagent',
    description: 'Spawned by another agent for subtask delegation',
  },
  {
    value: 'Interactive',
    label: 'Interactive',
    description: 'Real-time collaboration agent (e.g., pair programming)',
  },
];

export function SpawnAgentModal({
  isOpen,
  onClose,
  onSpawn,
  isSpawning,
}: SpawnAgentModalProps) {
  const [selectedType, setSelectedType] = useState<AgentType>('AdHoc');
  const [name, setName] = useState('');
  const [task, setTask] = useState('');

  if (!isOpen) return null;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSpawn(selectedType, name || undefined, task || undefined);
  };

  const handleBackdropClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget) {
      onClose();
    }
  };

  return (
    <div className="spawn-modal-backdrop" onClick={handleBackdropClick}>
      <div className="spawn-modal">
        <div className="spawn-modal-header">
          <h2 className="spawn-modal-title">SPAWN AGENT</h2>
          <button
            type="button"
            className="spawn-modal-close"
            onClick={onClose}
            aria-label="Close"
          >
            ×
          </button>
        </div>

        <form className="spawn-modal-form" onSubmit={handleSubmit}>
          <div className="spawn-modal-field">
            <label className="spawn-modal-label">Agent Type</label>
            <div className="spawn-modal-types">
              {agentTypes.map((type) => (
                <button
                  key={type.value}
                  type="button"
                  className={`spawn-modal-type ${
                    selectedType === type.value ? 'selected' : ''
                  }`}
                  onClick={() => setSelectedType(type.value)}
                >
                  <span className="spawn-modal-type-label">{type.label}</span>
                  <span className="spawn-modal-type-desc">{type.description}</span>
                </button>
              ))}
            </div>
          </div>

          <div className="spawn-modal-field">
            <label className="spawn-modal-label" htmlFor="agent-name">
              Name <span className="spawn-modal-optional">(optional)</span>
            </label>
            <input
              id="agent-name"
              type="text"
              className="spawn-modal-input"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="my-agent"
            />
          </div>

          <div className="spawn-modal-field">
            <label className="spawn-modal-label" htmlFor="agent-task">
              Initial Task <span className="spawn-modal-optional">(optional)</span>
            </label>
            <textarea
              id="agent-task"
              className="spawn-modal-textarea"
              value={task}
              onChange={(e) => setTask(e.target.value)}
              placeholder="Describe what this agent should do..."
              rows={3}
            />
          </div>

          <div className="spawn-modal-actions">
            <Button variant="secondary" onClick={onClose} disabled={isSpawning}>
              Cancel
            </Button>
            <Button variant="primary" type="submit" disabled={isSpawning}>
              {isSpawning ? 'Spawning...' : 'Spawn Agent'}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}
