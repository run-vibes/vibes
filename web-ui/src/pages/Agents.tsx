import { useState } from 'react';
import { Badge, Button, AgentCard } from '@vibes/design-system';
import { useAgents, useWebSocket } from '../hooks';
import { AgentInfo, getAgentStatusVariant } from '../lib/types';
import type { AgentStatusVariant, AgentTypeVariant } from '@vibes/design-system';
import { SpawnAgentModal } from '../components/SpawnAgentModal';
import { AgentDetailDrawer } from '../components/AgentDetailDrawer';
import './Agents.css';

function mapStatusToCardStatus(status: AgentInfo['status']): AgentStatusVariant {
  return getAgentStatusVariant(status);
}

function mapAgentType(type: AgentInfo['agent_type']): AgentTypeVariant {
  return type;
}

function getModelId(context: AgentInfo['context']): string {
  return context.model.id || 'unknown';
}

function getDurationSeconds(metrics?: AgentInfo['current_task_metrics']): number | undefined {
  if (!metrics) return undefined;
  return metrics.duration.secs + metrics.duration.nanos / 1e9;
}

export function AgentsPage() {
  const { send, addMessageHandler, isConnected, connectionState } = useWebSocket();
  const {
    agents,
    isLoading,
    isSpawning,
    error,
    refresh,
    spawnAgent,
    pauseAgent,
    resumeAgent,
    cancelAgent,
    stopAgent,
  } = useAgents({
    send,
    addMessageHandler,
    isConnected,
    autoRefresh: true,
    refreshInterval: 5000, // Refresh every 5 seconds for real-time updates
  });

  const [showSpawnModal, setShowSpawnModal] = useState(false);
  const [selectedAgent, setSelectedAgent] = useState<AgentInfo | null>(null);

  const handleSpawnAgent = async (
    type: AgentInfo['agent_type'],
    name?: string,
    task?: string
  ) => {
    try {
      await spawnAgent(type, name, task);
      setShowSpawnModal(false);
    } catch (err) {
      console.error('Failed to spawn agent:', err);
    }
  };

  const handleAgentAction = async (
    agentId: string,
    action: 'pause' | 'resume' | 'cancel' | 'stop'
  ) => {
    try {
      switch (action) {
        case 'pause':
          await pauseAgent(agentId);
          break;
        case 'resume':
          await resumeAgent(agentId);
          break;
        case 'cancel':
          await cancelAgent(agentId);
          break;
        case 'stop':
          await stopAgent(agentId);
          break;
      }
    } catch (err) {
      console.error(`Failed to ${action} agent:`, err);
    }
  };

  const getAgentActions = (agent: AgentInfo) => {
    const status = getAgentStatusVariant(agent.status);
    const actions = [];

    if (status === 'running') {
      actions.push({
        icon: '⏸',
        label: 'Pause agent',
        onClick: () => handleAgentAction(agent.id, 'pause'),
      });
      actions.push({
        icon: '✕',
        label: 'Cancel task',
        onClick: () => handleAgentAction(agent.id, 'cancel'),
      });
    } else if (status === 'paused') {
      actions.push({
        icon: '▶',
        label: 'Resume agent',
        onClick: () => handleAgentAction(agent.id, 'resume'),
      });
    }

    actions.push({
      icon: '⏻',
      label: 'Stop agent',
      onClick: () => handleAgentAction(agent.id, 'stop'),
    });

    return actions;
  };

  const getCurrentTask = (agent: AgentInfo): string | undefined => {
    const status = agent.status;
    // Unit variants (like 'idle') are strings, data variants are objects
    if (typeof status === 'string') return undefined;
    if ('running' in status) {
      return `Task ${status.running.task.slice(0, 8)}...`;
    }
    if ('paused' in status) {
      return status.paused.reason;
    }
    if ('waiting_for_input' in status) {
      return status.waiting_for_input.prompt;
    }
    if ('failed' in status) {
      return status.failed.error;
    }
    return undefined;
  };

  return (
    <div className="agents-page">
      {/* Header */}
      <div className="agents-header">
        <div className="agents-header-left">
          <h1 className="agents-title">AGENTS</h1>
          <div className="agents-status">
            {isConnected ? (
              <Badge status="success">Connected</Badge>
            ) : (
              <Badge status="error">{connectionState}</Badge>
            )}
          </div>
        </div>

        <div className="agents-header-right">
          <span className="agents-count">
            {agents.length} agent{agents.length !== 1 ? 's' : ''}
          </span>
          <Button
            variant="primary"
            size="sm"
            onClick={() => setShowSpawnModal(true)}
            disabled={isSpawning}
          >
            {isSpawning ? 'Spawning...' : '+ Spawn'}
          </Button>
        </div>
      </div>

      {/* Content */}
      <div className="agents-content">
        {isLoading && agents.length === 0 ? (
          <div className="agents-loading">Loading agents...</div>
        ) : error ? (
          <div className="agents-error">
            <div className="agents-error-text">Error: {error}</div>
            <Button variant="secondary" size="sm" onClick={refresh}>
              Retry
            </Button>
          </div>
        ) : agents.length === 0 ? (
          <div className="agents-empty">
            <div className="agents-empty-text">No active agents</div>
            <div className="agents-empty-hint">
              Spawn an agent to start autonomous task execution
            </div>
            <Button
              variant="primary"
              onClick={() => setShowSpawnModal(true)}
              disabled={isSpawning}
            >
              {isSpawning ? 'Spawning...' : 'Spawn Agent'}
            </Button>
          </div>
        ) : (
          <div className="agents-grid">
            {agents.map((agent) => (
              <AgentCard
                key={agent.id}
                id={agent.id}
                name={agent.name}
                agentType={mapAgentType(agent.agent_type)}
                status={mapStatusToCardStatus(agent.status)}
                currentTask={getCurrentTask(agent)}
                model={getModelId(agent.context)}
                tokensUsed={agent.current_task_metrics?.tokens_used}
                toolCalls={agent.current_task_metrics?.tool_calls}
                duration={getDurationSeconds(agent.current_task_metrics)}
                actions={getAgentActions(agent)}
                onClick={() => setSelectedAgent(agent)}
              />
            ))}
          </div>
        )}
      </div>

      {/* Spawn Modal */}
      {showSpawnModal && (
        <SpawnAgentModal
          isOpen={showSpawnModal}
          onClose={() => setShowSpawnModal(false)}
          onSpawn={handleSpawnAgent}
          isSpawning={isSpawning}
        />
      )}

      {/* Agent Detail Drawer */}
      {selectedAgent && (
        <AgentDetailDrawer
          agent={selectedAgent}
          onClose={() => setSelectedAgent(null)}
          onPause={() => handleAgentAction(selectedAgent.id, 'pause')}
          onResume={() => handleAgentAction(selectedAgent.id, 'resume')}
          onCancel={() => handleAgentAction(selectedAgent.id, 'cancel')}
          onStop={() => {
            handleAgentAction(selectedAgent.id, 'stop');
            setSelectedAgent(null);
          }}
        />
      )}
    </div>
  );
}
