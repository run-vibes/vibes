import { useCallback, useEffect, useState } from 'react';

export interface PushSubscriptionState {
  /** Whether push notifications are supported by the browser */
  isSupported: boolean;
  /** Whether the service worker is registered */
  isServiceWorkerReady: boolean;
  /** Whether the user has granted notification permission */
  hasPermission: boolean;
  /** Whether the user is currently subscribed */
  isSubscribed: boolean;
  /** The subscription ID if subscribed */
  subscriptionId: string | null;
  /** Whether an operation is in progress */
  isLoading: boolean;
  /** Error message if any */
  error: string | null;
}

const initialState: PushSubscriptionState = {
  isSupported: false,
  isServiceWorkerReady: false,
  hasPermission: false,
  isSubscribed: false,
  subscriptionId: null,
  isLoading: true,
  error: null,
};

// Local storage key for subscription ID
const SUBSCRIPTION_ID_KEY = 'vibes_push_subscription_id';

/**
 * Convert a base64url string to Uint8Array for the applicationServerKey
 */
function urlBase64ToUint8Array(base64String: string): Uint8Array {
  // Add padding if needed
  const padding = '='.repeat((4 - (base64String.length % 4)) % 4);
  const base64 = (base64String + padding)
    .replace(/-/g, '+')
    .replace(/_/g, '/');

  const rawData = window.atob(base64);
  const outputArray = new Uint8Array(rawData.length);

  for (let i = 0; i < rawData.length; ++i) {
    outputArray[i] = rawData.charCodeAt(i);
  }
  return outputArray;
}

/**
 * Hook to manage push notification subscription
 *
 * Handles:
 * - Checking browser support
 * - Registering service worker
 * - Requesting notification permission
 * - Subscribing/unsubscribing to push notifications
 * - Syncing subscription state with server
 */
export function usePushSubscription() {
  const [state, setState] = useState<PushSubscriptionState>(initialState);

  // Check browser support and initial state on mount
  useEffect(() => {
    const checkSupport = async () => {
      // Check if push notifications are supported
      const isSupported =
        'serviceWorker' in navigator &&
        'PushManager' in window &&
        'Notification' in window;

      if (!isSupported) {
        setState((prev) => ({
          ...prev,
          isSupported: false,
          isLoading: false,
        }));
        return;
      }

      // Check current permission
      const hasPermission = Notification.permission === 'granted';

      // Check if service worker is registered
      let isServiceWorkerReady = false;
      let isSubscribed = false;

      try {
        const registration = await navigator.serviceWorker.getRegistration();
        isServiceWorkerReady = !!registration;

        if (registration) {
          const subscription = await registration.pushManager.getSubscription();
          isSubscribed = !!subscription;
        }
      } catch (e) {
        console.error('Error checking service worker:', e);
      }

      // Get saved subscription ID
      const subscriptionId = localStorage.getItem(SUBSCRIPTION_ID_KEY);

      setState({
        isSupported: true,
        isServiceWorkerReady,
        hasPermission,
        isSubscribed,
        subscriptionId,
        isLoading: false,
        error: null,
      });
    };

    checkSupport();
  }, []);

  // Subscribe to push notifications
  const subscribe = useCallback(async () => {
    setState((prev) => ({ ...prev, isLoading: true, error: null }));

    try {
      // Check permission once - only request if not already granted
      const permission =
        Notification.permission === 'granted'
          ? Notification.permission
          : await Notification.requestPermission();

      if (permission !== 'granted') {
        setState((prev) => ({
          ...prev,
          hasPermission: false,
          isLoading: false,
          error: 'Notification permission denied',
        }));
        return;
      }

      // Register service worker if not registered
      let registration = await navigator.serviceWorker.getRegistration();
      if (!registration) {
        registration = await navigator.serviceWorker.register('/sw.js');
        // Wait for the service worker to be ready
        await navigator.serviceWorker.ready;
      }

      // Get VAPID public key from server
      const vapidResponse = await fetch('/api/push/vapid-key');
      if (!vapidResponse.ok) {
        throw new Error('Failed to get VAPID key from server');
      }
      const { public_key: vapidPublicKey } = await vapidResponse.json();

      // Subscribe to push notifications
      const applicationServerKey = urlBase64ToUint8Array(vapidPublicKey);
      const subscription = await registration.pushManager.subscribe({
        userVisibleOnly: true,
        applicationServerKey: applicationServerKey as BufferSource,
      });

      // Send subscription to server
      const subscriptionJson = subscription.toJSON();
      const response = await fetch('/api/push/subscribe', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          endpoint: subscriptionJson.endpoint,
          keys: {
            p256dh: subscriptionJson.keys?.p256dh,
            auth: subscriptionJson.keys?.auth,
          },
          user_agent: navigator.userAgent,
        }),
      });

      if (!response.ok) {
        throw new Error('Failed to save subscription on server');
      }

      const { id: subscriptionId } = await response.json();

      // Save subscription ID locally
      localStorage.setItem(SUBSCRIPTION_ID_KEY, subscriptionId);

      setState((prev) => ({
        ...prev,
        hasPermission: true,
        isServiceWorkerReady: true,
        isSubscribed: true,
        subscriptionId,
        isLoading: false,
        error: null,
      }));
    } catch (e) {
      console.error('Error subscribing to push:', e);
      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: e instanceof Error ? e.message : 'Failed to subscribe',
      }));
    }
  }, []);

  // Unsubscribe from push notifications
  const unsubscribe = useCallback(async () => {
    setState((prev) => ({ ...prev, isLoading: true, error: null }));

    try {
      // Get current subscription
      const registration = await navigator.serviceWorker.getRegistration();
      if (registration) {
        const subscription = await registration.pushManager.getSubscription();
        if (subscription) {
          await subscription.unsubscribe();
        }
      }

      // Remove subscription from server
      const subscriptionId = localStorage.getItem(SUBSCRIPTION_ID_KEY);
      if (subscriptionId) {
        await fetch(`/api/push/subscribe/${subscriptionId}`, {
          method: 'DELETE',
        });
        localStorage.removeItem(SUBSCRIPTION_ID_KEY);
      }

      setState((prev) => ({
        ...prev,
        isSubscribed: false,
        subscriptionId: null,
        isLoading: false,
        error: null,
      }));
    } catch (e) {
      console.error('Error unsubscribing:', e);
      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: e instanceof Error ? e.message : 'Failed to unsubscribe',
      }));
    }
  }, []);

  return {
    ...state,
    subscribe,
    unsubscribe,
  };
}
