// Vibes Push Notification Service Worker
// This service worker handles push notifications and click events

// Service Worker version for cache busting
const SW_VERSION = '1.0.0';

// Handle push notification events
self.addEventListener('push', (event) => {
  console.log('[SW] Push notification received');

  let data = {
    title: 'Vibes',
    body: 'You have a new notification',
    icon: '/vibes-icon.png',
    badge: '/vibes-badge.png',
    data: {},
  };

  // Try to parse the push data
  if (event.data) {
    try {
      const payload = event.data.json();

      // Map our notification types to display info
      switch (payload.event) {
        case 'permission_needed':
          data.title = 'Permission Required';
          data.body = payload.data?.message || 'Claude needs your approval';
          data.tag = `permission-${payload.data?.session_id}`;
          data.requireInteraction = true;
          break;
        case 'session_completed':
          data.title = 'Session Completed';
          data.body = payload.data?.message || 'Your Claude session has finished';
          data.tag = `completed-${payload.data?.session_id}`;
          break;
        case 'session_error':
          data.title = 'Session Error';
          data.body = payload.data?.message || 'An error occurred in your session';
          data.tag = `error-${payload.data?.session_id}`;
          data.requireInteraction = true;
          break;
        default:
          data.title = payload.title || data.title;
          data.body = payload.body || data.body;
      }

      // Store session_id for click handling
      if (payload.data?.session_id) {
        data.data.session_id = payload.data.session_id;
      }
      data.data.event = payload.event;
    } catch (e) {
      console.warn('[SW] Failed to parse push data:', e);
      // Try to use the text directly
      const text = event.data.text();
      if (text) {
        data.body = text;
      }
    }
  }

  // Show the notification
  event.waitUntil(
    self.registration.showNotification(data.title, {
      body: data.body,
      icon: data.icon,
      badge: data.badge,
      tag: data.tag,
      requireInteraction: data.requireInteraction,
      data: data.data,
    })
  );
});

// Handle notification click events
self.addEventListener('notificationclick', (event) => {
  console.log('[SW] Notification clicked');

  event.notification.close();

  const data = event.notification.data || {};
  let targetUrl = '/';

  // Navigate to the relevant session if we have a session_id
  if (data.session_id) {
    targetUrl = `/sessions/${data.session_id}`;
  }

  // Focus existing window or open new one
  event.waitUntil(
    clients.matchAll({ type: 'window', includeUncontrolled: true })
      .then((windowClients) => {
        // Check if there's already a window we can focus
        for (const client of windowClients) {
          if (client.url.includes(self.location.origin) && 'focus' in client) {
            return client.focus().then((focusedClient) => {
              // Navigate to the target URL
              if (focusedClient.navigate) {
                return focusedClient.navigate(targetUrl);
              }
            });
          }
        }
        // No existing window, open a new one
        if (clients.openWindow) {
          return clients.openWindow(targetUrl);
        }
      })
  );
});

// Handle service worker installation
self.addEventListener('install', (event) => {
  console.log('[SW] Service worker installed, version:', SW_VERSION);
  // Skip waiting to activate immediately
  self.skipWaiting();
});

// Handle service worker activation
self.addEventListener('activate', (event) => {
  console.log('[SW] Service worker activated, version:', SW_VERSION);
  // Take control of all clients immediately
  event.waitUntil(clients.claim());
});
