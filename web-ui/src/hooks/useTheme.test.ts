/**
 * Tests for the useTheme hook and ThemeProvider
 *
 * Tests cover:
 * - Context initialization
 * - Theme toggling
 * - localStorage persistence
 * - System preference detection
 * - Keyboard shortcut
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { createElement, ReactNode } from 'react';
import { ThemeProvider, useTheme } from './useTheme';

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string) => store[key] || null),
    setItem: vi.fn((key: string, value: string) => {
      store[key] = value;
    }),
    removeItem: vi.fn((key: string) => {
      delete store[key];
    }),
    clear: vi.fn(() => {
      store = {};
    }),
  };
})();

Object.defineProperty(window, 'localStorage', { value: localStorageMock });

// Mock matchMedia
const createMatchMediaMock = (prefersDark: boolean) => {
  const listeners: Array<(e: MediaQueryListEvent) => void> = [];
  return vi.fn().mockImplementation((query: string) => ({
    matches: query === '(prefers-color-scheme: dark)' ? prefersDark : !prefersDark,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn((_: string, cb: (e: MediaQueryListEvent) => void) => {
      listeners.push(cb);
    }),
    removeEventListener: vi.fn((_: string, cb: (e: MediaQueryListEvent) => void) => {
      const idx = listeners.indexOf(cb);
      if (idx > -1) listeners.splice(idx, 1);
    }),
    dispatchEvent: vi.fn(),
    // For testing: allow triggering change events
    _triggerChange: (matches: boolean) => {
      listeners.forEach(cb => cb({ matches } as MediaQueryListEvent));
    },
  }));
};

// Wrapper for renderHook
const wrapper = ({ children }: { children: ReactNode }) =>
  createElement(ThemeProvider, null, children);

describe('useTheme', () => {
  beforeEach(() => {
    localStorageMock.clear();
    vi.clearAllMocks();
    // Default to dark system preference
    window.matchMedia = createMatchMediaMock(true);
    // Reset document theme
    delete document.documentElement.dataset.theme;
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('initialization', () => {
    it('throws when used outside ThemeProvider', () => {
      expect(() => renderHook(() => useTheme())).toThrow(
        'useTheme must be used within a ThemeProvider'
      );
    });

    it('defaults to dark when no stored preference and system prefers dark', () => {
      window.matchMedia = createMatchMediaMock(true);
      const { result } = renderHook(() => useTheme(), { wrapper });
      expect(result.current.theme).toBe('dark');
    });

    it('defaults to light when no stored preference and system prefers light', () => {
      window.matchMedia = createMatchMediaMock(false);
      const { result } = renderHook(() => useTheme(), { wrapper });
      expect(result.current.theme).toBe('light');
    });

    it('uses stored preference over system preference', () => {
      window.matchMedia = createMatchMediaMock(true); // system prefers dark
      localStorageMock.setItem('vibes-theme', 'light'); // stored light
      const { result } = renderHook(() => useTheme(), { wrapper });
      expect(result.current.theme).toBe('light');
    });

    it('ignores invalid stored values', () => {
      localStorageMock.setItem('vibes-theme', 'invalid');
      window.matchMedia = createMatchMediaMock(false); // system prefers light
      const { result } = renderHook(() => useTheme(), { wrapper });
      expect(result.current.theme).toBe('light');
    });
  });

  describe('toggleTheme', () => {
    it('toggles from dark to light', () => {
      localStorageMock.setItem('vibes-theme', 'dark');
      const { result } = renderHook(() => useTheme(), { wrapper });

      act(() => {
        result.current.toggleTheme();
      });

      expect(result.current.theme).toBe('light');
    });

    it('toggles from light to dark', () => {
      localStorageMock.setItem('vibes-theme', 'light');
      const { result } = renderHook(() => useTheme(), { wrapper });

      act(() => {
        result.current.toggleTheme();
      });

      expect(result.current.theme).toBe('dark');
    });
  });

  describe('setTheme', () => {
    it('sets theme to specified value', () => {
      const { result } = renderHook(() => useTheme(), { wrapper });

      act(() => {
        result.current.setTheme('light');
      });

      expect(result.current.theme).toBe('light');

      act(() => {
        result.current.setTheme('dark');
      });

      expect(result.current.theme).toBe('dark');
    });
  });

  describe('persistence', () => {
    it('saves theme to localStorage on change', () => {
      const { result } = renderHook(() => useTheme(), { wrapper });

      act(() => {
        result.current.setTheme('light');
      });

      expect(localStorageMock.setItem).toHaveBeenCalledWith('vibes-theme', 'light');
    });

    it('updates document.documentElement.dataset.theme', () => {
      const { result } = renderHook(() => useTheme(), { wrapper });

      act(() => {
        result.current.setTheme('light');
      });

      expect(document.documentElement.dataset.theme).toBe('light');
    });
  });

  describe('keyboard shortcut', () => {
    it('toggles theme on Ctrl+Shift+T (Windows/Linux)', () => {
      localStorageMock.setItem('vibes-theme', 'dark');
      Object.defineProperty(navigator, 'platform', { value: 'Win32', configurable: true });

      const { result } = renderHook(() => useTheme(), { wrapper });

      act(() => {
        document.dispatchEvent(
          new KeyboardEvent('keydown', {
            key: 'T',
            ctrlKey: true,
            shiftKey: true,
          })
        );
      });

      expect(result.current.theme).toBe('light');
    });

    it('toggles theme on Cmd+Shift+T (Mac)', () => {
      localStorageMock.setItem('vibes-theme', 'dark');
      Object.defineProperty(navigator, 'platform', { value: 'MacIntel', configurable: true });

      const { result } = renderHook(() => useTheme(), { wrapper });

      act(() => {
        document.dispatchEvent(
          new KeyboardEvent('keydown', {
            key: 'T',
            metaKey: true,
            shiftKey: true,
          })
        );
      });

      expect(result.current.theme).toBe('light');
    });

    it('does not toggle without shift key', () => {
      localStorageMock.setItem('vibes-theme', 'dark');
      Object.defineProperty(navigator, 'platform', { value: 'Win32', configurable: true });

      const { result } = renderHook(() => useTheme(), { wrapper });

      act(() => {
        document.dispatchEvent(
          new KeyboardEvent('keydown', {
            key: 'T',
            ctrlKey: true,
            shiftKey: false,
          })
        );
      });

      expect(result.current.theme).toBe('dark');
    });

    it('does not toggle without modifier key', () => {
      localStorageMock.setItem('vibes-theme', 'dark');

      const { result } = renderHook(() => useTheme(), { wrapper });

      act(() => {
        document.dispatchEvent(
          new KeyboardEvent('keydown', {
            key: 'T',
            shiftKey: true,
          })
        );
      });

      expect(result.current.theme).toBe('dark');
    });
  });

  describe('stable references', () => {
    it('toggleTheme has stable reference', () => {
      const { result, rerender } = renderHook(() => useTheme(), { wrapper });
      const toggleRef = result.current.toggleTheme;

      act(() => {
        result.current.toggleTheme();
      });

      rerender();
      expect(result.current.toggleTheme).toBe(toggleRef);
    });

    it('setTheme has stable reference', () => {
      const { result, rerender } = renderHook(() => useTheme(), { wrapper });
      const setRef = result.current.setTheme;

      act(() => {
        result.current.setTheme('light');
      });

      rerender();
      expect(result.current.setTheme).toBe(setRef);
    });
  });
});
