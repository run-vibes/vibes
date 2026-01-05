import { usePushSubscription } from '../hooks/usePushSubscription';

const styles = {
  container: {
    padding: 'var(--space-4)',
    backgroundColor: 'var(--surface)',
    borderRadius: 'var(--radius-md)',
    marginBottom: 'var(--space-4)',
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: 'var(--space-3)',
  },
  title: {
    fontSize: 'var(--font-size-base)',
    fontWeight: 600,
    color: 'var(--text)',
    margin: 0,
  },
  description: {
    fontSize: 'var(--font-size-sm)',
    color: 'var(--text-dim)',
    marginBottom: 'var(--space-4)',
  },
  button: {
    padding: 'var(--space-2) var(--space-4)',
    borderRadius: 'var(--radius-md)',
    border: 'none',
    fontSize: 'var(--font-size-sm)',
    fontWeight: 500,
    cursor: 'pointer',
    transition: 'background-color var(--transition-fast)',
  },
  enableButton: {
    backgroundColor: 'var(--phosphor)',
    color: 'var(--screen)',
  },
  disableButton: {
    backgroundColor: 'var(--surface-light)',
    color: 'var(--text)',
  },
  disabledButton: {
    backgroundColor: 'var(--surface-light)',
    color: 'var(--text-faint)',
    cursor: 'not-allowed',
  },
  status: {
    display: 'flex',
    alignItems: 'center',
    gap: 'var(--space-2)',
    fontSize: 'var(--font-size-sm)',
    color: 'var(--text-dim)',
  },
  statusIcon: {
    fontSize: '0.75rem',
  },
  error: {
    marginTop: 'var(--space-2)',
    padding: 'var(--space-2)',
    backgroundColor: 'var(--red-subtle)',
    borderRadius: 'var(--radius-sm)',
    color: 'var(--red)',
    fontSize: 'var(--font-size-sm)',
  },
  notSupported: {
    color: 'var(--text-faint)',
    fontSize: 'var(--font-size-sm)',
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
      return { icon: '◐', color: 'var(--status-starting)', text: 'Loading...' };
    }
    if (isSubscribed) {
      return { icon: '●', color: 'var(--status-connected)', text: 'Enabled' };
    }
    if (!hasPermission && Notification.permission === 'denied') {
      return { icon: '●', color: 'var(--status-failed)', text: 'Blocked by browser' };
    }
    return { icon: '○', color: 'var(--status-disabled)', text: 'Disabled' };
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
