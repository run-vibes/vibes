import { createContext, useContext, useState, useEffect, useCallback, ReactNode, createElement } from 'react';

export interface CrtEffectsContextValue {
  enabled: boolean;
  toggleEffects: () => void;
  setEffects: (enabled: boolean) => void;
}

const STORAGE_KEY = 'vibes-crt-effects';

/**
 * Get the initial CRT effects state from localStorage.
 * Defaults to enabled (true).
 */
function getInitialEffectsState(): boolean {
  if (typeof window === 'undefined') return true;

  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === 'false') {
    return false;
  }

  // Default to enabled
  return true;
}

const CrtEffectsContext = createContext<CrtEffectsContextValue | null>(null);

export interface CrtEffectsProviderProps {
  children: ReactNode;
}

/**
 * Provides CRT effects context to the application.
 * Handles:
 * - localStorage persistence
 * - Toggle functionality
 */
export function CrtEffectsProvider({ children }: CrtEffectsProviderProps) {
  const [enabled, setEnabledState] = useState<boolean>(getInitialEffectsState);

  // Sync effects state to localStorage
  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, String(enabled));
  }, [enabled]);

  const toggleEffects = useCallback(() => {
    setEnabledState((current) => !current);
  }, []);

  const setEffects = useCallback((newEnabled: boolean) => {
    setEnabledState(newEnabled);
  }, []);

  const value: CrtEffectsContextValue = {
    enabled,
    toggleEffects,
    setEffects,
  };

  // Wrap children in a div that conditionally applies crt-effects class
  return createElement(
    CrtEffectsContext.Provider,
    { value },
    createElement(
      'div',
      { className: enabled ? 'crt-effects' : undefined },
      children
    )
  );
}

/**
 * Hook to access CRT effects context.
 * Must be used within a CrtEffectsProvider.
 */
export function useCrtEffects(): CrtEffectsContextValue {
  const context = useContext(CrtEffectsContext);
  if (!context) {
    throw new Error('useCrtEffects must be used within a CrtEffectsProvider');
  }
  return context;
}
