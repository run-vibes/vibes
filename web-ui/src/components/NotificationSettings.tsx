import { usePushSubscription } from '../hooks/usePushSubscription';

const styles = {
  container: {
    padding: '1rem',
    backgroundColor: '#1f2937',
    borderRadius: '0.5rem',
    marginBottom: '1rem',
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: '0.75rem',
  },
  title: {
    fontSize: '1rem',
    fontWeight: 600,
    color: '#f9fafb',
    margin: 0,
  },
  description: {
    fontSize: '0.875rem',
    color: '#9ca3af',
    marginBottom: '1rem',
  },
  button: {
    padding: '0.5rem 1rem',
    borderRadius: '0.375rem',
    border: 'none',
    fontSize: '0.875rem',
    fontWeight: 500,
    cursor: 'pointer',
    transition: 'background-color 0.15s',
  },
  enableButton: {
    backgroundColor: '#3b82f6',
    color: 'white',
  },
  disableButton: {
    backgroundColor: '#374151',
    color: '#d1d5db',
  },
  disabledButton: {
    backgroundColor: '#374151',
    color: '#6b7280',
    cursor: 'not-allowed',
  },
  status: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
    fontSize: '0.875rem',
    color: '#9ca3af',
  },
  statusIcon: {
    fontSize: '0.75rem',
  },
  error: {
    marginTop: '0.5rem',
    padding: '0.5rem',
    backgroundColor: '#7f1d1d',
    borderRadius: '0.25rem',
    color: '#fca5a5',
    fontSize: '0.875rem',
  },
  notSupported: {
    color: '#6b7280',
    fontSize: '0.875rem',
  },
} as const;

export function NotificationSettings() {
  const {
    isSupported,
    isSubscribed,
    isLoading,
    hasPermission,
    error,
    subscribe,
    unsubscribe,
  } = usePushSubscription();

  // Not supported in this browser
  if (!isSupported) {
    return (
      <div style={styles.container}>
        <h3 style={styles.title}>Push Notifications</h3>
        <p style={styles.notSupported}>
          Push notifications are not supported in this browser.
        </p>
      </div>
    );
  }

  const handleToggle = async () => {
    if (isSubscribed) {
      await unsubscribe();
    } else {
      await subscribe();
    }
  };

  const getStatusInfo = () => {
    if (isLoading) {
      return { icon: '◐', color: '#f59e0b', text: 'Loading...' };
    }
    if (isSubscribed) {
      return { icon: '●', color: '#10b981', text: 'Enabled' };
    }
    if (!hasPermission && Notification.permission === 'denied') {
      return { icon: '●', color: '#ef4444', text: 'Blocked by browser' };
    }
    return { icon: '○', color: '#9ca3af', text: 'Disabled' };
  };

  const statusInfo = getStatusInfo();
  const isBlocked = Notification.permission === 'denied';

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h3 style={styles.title}>Push Notifications</h3>
        <div style={styles.status}>
          <span style={{ ...styles.statusIcon, color: statusInfo.color }}>
            {statusInfo.icon}
          </span>
          <span>{statusInfo.text}</span>
        </div>
      </div>

      <p style={styles.description}>
        Get notified when Claude needs permission, sessions complete, or errors occur.
      </p>

      {isBlocked ? (
        <p style={styles.notSupported}>
          Notifications are blocked. Please enable them in your browser settings.
        </p>
      ) : (
        <button
          onClick={handleToggle}
          disabled={isLoading}
          style={{
            ...styles.button,
            ...(isLoading
              ? styles.disabledButton
              : isSubscribed
                ? styles.disableButton
                : styles.enableButton),
          }}
        >
          {isLoading
            ? 'Please wait...'
            : isSubscribed
              ? 'Disable Notifications'
              : 'Enable Notifications'}
        </button>
      )}

      {error && (
        <div style={styles.error}>
          {error}
        </div>
      )}
    </div>
  );
}
