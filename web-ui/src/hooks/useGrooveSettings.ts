import { useState, useCallback } from 'react';

const STORAGE_KEY = 'vibes:groove-settings';

export interface GrooveSettings {
  showLearningIndicator: boolean;
  dashboardAutoRefresh: boolean;
}

const DEFAULT_SETTINGS: GrooveSettings = {
  showLearningIndicator: false,
  dashboardAutoRefresh: true,
};

function loadSettings(): GrooveSettings {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      return { ...DEFAULT_SETTINGS, ...parsed };
    }
  } catch {
    // Invalid JSON, use defaults
  }
  return DEFAULT_SETTINGS;
}

function saveSettings(settings: GrooveSettings): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
}

export function useGrooveSettings() {
  const [settings, setSettings] = useState<GrooveSettings>(loadSettings);

  const updateSetting = useCallback(
    <K extends keyof GrooveSettings>(key: K, value: GrooveSettings[K]) => {
      setSettings((prev) => {
        const next = { ...prev, [key]: value };
        saveSettings(next);
        return next;
      });
    },
    []
  );

  return { settings, updateSetting };
}
