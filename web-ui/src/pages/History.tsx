import { useState } from 'react';
import { HistoryList } from '../components/HistoryList';
import { HistoryDetail } from '../components/HistoryDetail';

export function HistoryPage() {
  const [selectedSessionId, setSelectedSessionId] = useState<string | null>(null);

  const handleResume = (claudeSessionId: string) => {
    // Navigate to active session or trigger resume flow
    console.log('Resume with Claude session:', claudeSessionId);
    // TODO: Integrate with session management
  };

  return (
    <div className="history-page">
      <h1>Chat History</h1>
      {selectedSessionId ? (
        <HistoryDetail
          sessionId={selectedSessionId}
          onBack={() => setSelectedSessionId(null)}
          onResume={handleResume}
        />
      ) : (
        <HistoryList onSelectSession={setSelectedSessionId} />
      )}
    </div>
  );
}
