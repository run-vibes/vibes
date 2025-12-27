import { useQuery } from '@tanstack/react-query';
import { getSession, getSessionMessages, resumeSession, HistoricalMessage } from '../api/history';
import { useState } from 'react';

interface HistoryDetailProps {
  sessionId: string;
  onBack: () => void;
  onResume?: (claudeSessionId: string) => void;
}

export function HistoryDetail({ sessionId, onBack, onResume }: HistoryDetailProps) {
  const [resumeError, setResumeError] = useState<string | null>(null);
  const [isResuming, setIsResuming] = useState(false);

  const { data: session, isLoading: sessionLoading } = useQuery({
    queryKey: ['history-session', sessionId],
    queryFn: () => getSession(sessionId),
  });

  const { data: messages, isLoading: messagesLoading } = useQuery({
    queryKey: ['history-messages', sessionId],
    queryFn: () => getSessionMessages(sessionId, { limit: 500 }),
  });

  const handleResume = async () => {
    setResumeError(null);
    setIsResuming(true);
    try {
      const result = await resumeSession(sessionId);
      if (result.claude_session_id && onResume) {
        onResume(result.claude_session_id);
      }
    } catch (e) {
      setResumeError(e instanceof Error ? e.message : 'Failed to resume');
    } finally {
      setIsResuming(false);
    }
  };

  const formatDate = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString();
  };

  const renderMessage = (msg: HistoricalMessage) => {
    const roleClass = msg.role.replace('_', '-');
    return (
      <div key={msg.id} className={`message message-${roleClass}`}>
        <div className="message-header">
          <span className="message-role">{msg.role}</span>
          {msg.tool_name && (
            <span className="message-tool">{msg.tool_name}</span>
          )}
          <span className="message-time">{formatDate(msg.created_at)}</span>
        </div>
        <div className="message-content">
          {msg.role === 'tool_use' || msg.role === 'tool_result' ? (
            <pre>{msg.content}</pre>
          ) : (
            <p>{msg.content}</p>
          )}
        </div>
      </div>
    );
  };

  if (sessionLoading) {
    return <div className="loading">Loading session...</div>;
  }

  if (!session) {
    return <div className="error">Session not found</div>;
  }

  return (
    <div className="history-detail">
      <div className="detail-header">
        <button onClick={onBack} className="back-btn">
          &larr; Back to History
        </button>

        <h2>{session.name || 'Unnamed Session'}</h2>

        <div className="session-info">
          <span className={`state ${session.state.toLowerCase()}`}>
            {session.state}
          </span>
          <span>Created: {formatDate(session.created_at)}</span>
          <span>Messages: {session.message_count}</span>
          <span>
            Tokens: {session.total_input_tokens} in / {session.total_output_tokens} out
          </span>
        </div>

        {session.claude_session_id && onResume && (
          <div className="resume-section">
            <button
              onClick={handleResume}
              disabled={isResuming}
              className="resume-btn"
            >
              {isResuming ? 'Resuming...' : 'Resume Session'}
            </button>
            {resumeError && (
              <div className="resume-error">{resumeError}</div>
            )}
          </div>
        )}

        {session.error_message && (
          <div className="error-message">
            Error: {session.error_message}
          </div>
        )}
      </div>

      <div className="messages-container">
        {messagesLoading ? (
          <div className="loading">Loading messages...</div>
        ) : messages?.messages.length === 0 ? (
          <div className="empty">No messages</div>
        ) : (
          <div className="messages-list">
            {messages?.messages.map(renderMessage)}
          </div>
        )}
      </div>
    </div>
  );
}
