import { describe, expect, it } from 'vitest';
import {
  comboFromEvent,
  comboMatches,
  formatCombo,
  getShortcut,
  SHORTCUT_ACTIONS,
} from '../shortcuts';

const event = (init: Partial<KeyboardEvent>): KeyboardEvent =>
  ({
    key: '',
    metaKey: false,
    ctrlKey: false,
    altKey: false,
    shiftKey: false,
    ...init,
  }) as KeyboardEvent;

describe('shortcuts', () => {
  it('has unique action ids', () => {
    expect(new Set(SHORTCUT_ACTIONS.map((a) => a.id)).size).toBe(SHORTCUT_ACTIONS.length);
  });

  it('returns defaults without overrides', () => {
    expect(getShortcut('save')).toBe('mod+s');
    expect(getShortcut('run-all')).toBe('f6');
  });

  it('matches combos against keyboard events', () => {
    expect(comboMatches('mod+s', event({ key: 's', metaKey: true }))).toBe(true);
    expect(comboMatches('mod+s', event({ key: 's', ctrlKey: true }))).toBe(true);
    expect(comboMatches('mod+s', event({ key: 's' }))).toBe(false);
    expect(comboMatches('shift+enter', event({ key: 'Enter', shiftKey: true }))).toBe(true);
    expect(comboMatches('shift+enter', event({ key: 'Enter' }))).toBe(false);
    expect(comboMatches('f6', event({ key: 'F6' }))).toBe(true);
    expect(comboMatches('a', event({ key: 'a' }))).toBe(true);
    expect(comboMatches('a', event({ key: 'a', shiftKey: true }))).toBe(false);
  });

  it('builds combos from events', () => {
    expect(comboFromEvent(event({ key: 'S', metaKey: true }))).toBe('mod+s');
    expect(comboFromEvent(event({ key: 'Enter', shiftKey: true }))).toBe('shift+enter');
  });

  it('formats combos for display', () => {
    expect(formatCombo('mod+shift+z')).toBe('⌘/Ctrl + Shift + Z');
  });
});
