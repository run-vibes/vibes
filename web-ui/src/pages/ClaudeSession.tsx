import { useParams, Link } from '@tanstack/react-router';
import { useEffect, useState, useCallback } from 'react';
import { useWebSocket } from '../hooks/useWebSocket';
import type { ServerMessage, ClaudeEvent, SessionState, InputSource } from '../lib/types';

interface ConversationItem {
  type: 'user' | 'assistant' | 'tool';
  content: string;
  timestamp: Date;
  source?: InputSource;
  isOwn?: boolean;
}

function getSourceLabel(source: InputSource): string {
  switch (source) {
    case 'cli': return '📟 CLI';
    case 'web_ui': return '🌐 Web';
    default: return '❓ Unknown';
  }
}

export function ClaudeSession() {
  const { sessionId } = useParams({ from: '/claude/$sessionId' });
  const { isConnected, subscribe, addMessageHandler, send } = useWebSocket();

  const [state, setState] = useState<SessionState>('idle');
  const [conversation, setConversation] = useState<ConversationItem[]>([]);
  const [streamingText, setStreamingText] = useState('');
  const [input, setInput] = useState('');
  const [pendingPermission, setPendingPermission] = useState<{
    id: string;
    tool: string;
    description: string;
  } | null>(null);

  // Subscribe to session events
  useEffect(() => {
    if (isConnected) {
      subscribe([sessionId]);
    }
  }, [isConnected, sessionId, subscribe]);

  // Handle incoming messages
  const handleMessage = useCallback((msg: ServerMessage) => {
    if (msg.type === 'session_state' && msg.session_id === sessionId) {
      setState(msg.state);
    }

    if (msg.type === 'claude' && msg.session_id === sessionId) {
      const event = msg.event;
      handleClaudeEvent(event);
    }

    // Handle remote user input (from CLI or other web sessions)
    if (msg.type === 'user_input' && msg.session_id === sessionId && msg.source !== 'web_ui') {
      setConversation(prev => [...prev, {
        type: 'user',
        content: msg.content,
        timestamp: new Date(),
        source: msg.source,
        isOwn: false,
      }]);
    }
  }, [sessionId]);

  const handleClaudeEvent = (event: ClaudeEvent) => {
    switch (event.type) {
      case 'text_delta':
        setStreamingText(prev => prev + event.text);
        break;
      case 'turn_start':
        setStreamingText('');
        break;
      case 'turn_complete':
        if (streamingText) {
          setConversation(prev => [...prev, {
            type: 'assistant',
            content: streamingText,
            timestamp: new Date(),
          }]);
          setStreamingText('');
        }
        break;
      case 'permission_request':
        setPendingPermission({
          id: event.id,
          tool: event.tool,
          description: event.description,
        });
        break;
      case 'error':
        setConversation(prev => [...prev, {
          type: 'assistant',
          content: `Error: ${event.message}`,
          timestamp: new Date(),
        }]);
        break;
    }
  };

  useEffect(() => {
    return addMessageHandler(handleMessage);
  }, [addMessageHandler, handleMessage]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim()) return;

    setConversation(prev => [...prev, {
      type: 'user',
      content: input,
      timestamp: new Date(),
      source: 'web_ui',
      isOwn: true,
    }]);

    send({
      type: 'input',
      session_id: sessionId,
      content: input,
    });

    setInput('');
  };

  const handlePermission = (approved: boolean) => {
    if (!pendingPermission) return;

    send({
      type: 'permission_response',
      session_id: sessionId,
      request_id: pendingPermission.id,
      approved,
    });

    setPendingPermission(null);
  };

  return (
    <div className="page session-detail">
      <div className="session-header">
        <Link to="/claude" className="back-link">&larr; Sessions</Link>
        <h1>Session {sessionId.slice(0, 8)}</h1>
        <span className={`status status-${state}`}>{state}</span>
      </div>

      {!isConnected && (
        <div className="connection-status">
          Connecting to daemon...
        </div>
      )}

      {pendingPermission && (
        <div className="permission-card">
          <h3>Permission Required</h3>
          <p><strong>{pendingPermission.tool}</strong></p>
          <p>{pendingPermission.description}</p>
          <div className="permission-actions">
            <button onClick={() => handlePermission(true)} className="button approve">
              Allow
            </button>
            <button onClick={() => handlePermission(false)} className="button deny">
              Deny
            </button>
          </div>
        </div>
      )}

      <div className="conversation">
        {conversation.map((item, i) => (
          <div key={i} className={`message message-${item.type} ${item.isOwn === false ? 'message-remote' : ''}`}>
            {item.type === 'user' && item.isOwn === false && item.source && (
              <div className="message-source">{getSourceLabel(item.source)}</div>
            )}
            <div className="message-content">{item.content}</div>
          </div>
        ))}
        {streamingText && (
          <div className="message message-assistant streaming">
            <div className="message-content">{streamingText}</div>
          </div>
        )}
      </div>

      <form onSubmit={handleSubmit} className="input-form">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Send a message..."
          disabled={state === 'processing'}
        />
        <button type="submit" disabled={state === 'processing' || !input.trim()}>
          Send
        </button>
      </form>
    </div>
  );
}
