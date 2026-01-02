import { describe, test, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { SettingsPage } from './Settings';

describe('SettingsPage', () => {
  describe('About section', () => {
    test('GitHub link points to run-vibes/vibes', () => {
      render(<SettingsPage />);

      const githubLink = screen.getByRole('link', { name: /github/i });
      expect(githubLink).toHaveAttribute('href', 'https://github.com/run-vibes/vibes');
    });

    test('Report Issue link points to run-vibes/vibes/issues', () => {
      render(<SettingsPage />);

      const issueLink = screen.getByRole('link', { name: /report issue/i });
      expect(issueLink).toHaveAttribute('href', 'https://github.com/run-vibes/vibes/issues');
    });
  });
});
