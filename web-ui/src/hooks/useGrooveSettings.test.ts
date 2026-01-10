/**
 * Tests for useGrooveSettings hook
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useGrooveSettings } from './useGrooveSettings';

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string) => store[key] || null),
    setItem: vi.fn((key: string, value: string) => {
      store[key] = value;
    }),
    clear: vi.fn(() => {
      store = {};
    }),
  };
})();

Object.defineProperty(window, 'localStorage', { value: localStorageMock });

describe('useGrooveSettings', () => {
  beforeEach(() => {
    localStorageMock.clear();
    vi.clearAllMocks();
  });

  it('returns default settings', () => {
    const { result } = renderHook(() => useGrooveSettings());

    expect(result.current.settings.showLearningIndicator).toBe(false);
    expect(result.current.settings.dashboardAutoRefresh).toBe(true);
  });

  it('loads settings from localStorage', () => {
    localStorageMock.getItem.mockReturnValue(
      JSON.stringify({
        showLearningIndicator: true,
        dashboardAutoRefresh: false,
      })
    );

    const { result } = renderHook(() => useGrooveSettings());

    expect(result.current.settings.showLearningIndicator).toBe(true);
    expect(result.current.settings.dashboardAutoRefresh).toBe(false);
  });

  it('updates showLearningIndicator setting', () => {
    const { result } = renderHook(() => useGrooveSettings());

    act(() => {
      result.current.updateSetting('showLearningIndicator', true);
    });

    expect(result.current.settings.showLearningIndicator).toBe(true);
  });

  it('updates dashboardAutoRefresh setting', () => {
    const { result } = renderHook(() => useGrooveSettings());

    act(() => {
      result.current.updateSetting('dashboardAutoRefresh', false);
    });

    expect(result.current.settings.dashboardAutoRefresh).toBe(false);
  });

  it('persists settings to localStorage', () => {
    const { result } = renderHook(() => useGrooveSettings());

    act(() => {
      result.current.updateSetting('showLearningIndicator', true);
    });

    expect(localStorageMock.setItem).toHaveBeenCalledWith(
      'vibes:groove-settings',
      expect.stringContaining('"showLearningIndicator":true')
    );
  });

  it('handles invalid JSON in localStorage', () => {
    localStorageMock.getItem.mockReturnValue('invalid json');

    const { result } = renderHook(() => useGrooveSettings());

    // Should fall back to defaults
    expect(result.current.settings.showLearningIndicator).toBe(false);
    expect(result.current.settings.dashboardAutoRefresh).toBe(true);
  });

  it('preserves other settings when updating one', () => {
    const { result } = renderHook(() => useGrooveSettings());

    act(() => {
      result.current.updateSetting('showLearningIndicator', true);
    });

    act(() => {
      result.current.updateSetting('dashboardAutoRefresh', false);
    });

    expect(result.current.settings.showLearningIndicator).toBe(true);
    expect(result.current.settings.dashboardAutoRefresh).toBe(false);
  });
});
