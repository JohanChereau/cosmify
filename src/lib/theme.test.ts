import { afterEach, describe, expect, it } from 'vitest';
import { applyTheme } from './theme';

afterEach(() => {
  delete document.documentElement.dataset.theme;
});

describe('applyTheme', () => {
  it('applies explicit light and dark themes', () => {
    applyTheme('dark');
    expect(document.documentElement.dataset.theme).toBe('dark');
    applyTheme('light');
    expect(document.documentElement.dataset.theme).toBe('light');
  });

  it('falls back to the system theme for unknown or system values', () => {
    document.documentElement.dataset.theme = 'dark';
    applyTheme('system');
    expect(document.documentElement.dataset.theme).toBeUndefined();
    document.documentElement.dataset.theme = 'light';
    applyTheme('unexpected');
    expect(document.documentElement.dataset.theme).toBeUndefined();
  });
});
