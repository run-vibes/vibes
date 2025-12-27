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

      // Use title and body directly from payload (matches Rust PushNotification type)
      data.title = payload.title || data.title;
      data.body = payload.body || data.body;
      data.tag = payload.tag || data.tag;

      // Check event_type from nested data object
      const eventType = payload.data?.event_type;
      switch (eventType) {
        case 'permission_needed':
          data.requireInteraction = true;
          break;
        case 'session_error':
          data.requireInteraction = true;
          break;
        // session_completed and others don't require interaction
      }

      // Store session_id and event_type for click handling
      if (payload.data?.session_id) {
        data.data.session_id = payload.data.session_id;
      }
      if (payload.data?.url) {
        data.data.url = payload.data.url;
      }
      data.data.event_type = eventType;
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
  // Use URL from notification data, or fallback to session URL, or home
  let targetUrl = data.url || (data.session_id ? `/sessions/${data.session_id}` : '/');

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
