import { beforeEach, describe, expect, it } from 'vitest';
import {
  comboFromEvent,
  comboMatches,
  formatCombo,
  getShortcut,
  resetShortcuts,
  setShortcut,
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
  beforeEach(() => window.localStorage.clear());

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

  it('returns an empty string for unknown shortcut ids', () => {
    expect(getShortcut('does-not-exist')).toBe('');
  });

  it('applies and clears shortcut overrides', () => {
    setShortcut('save', 'mod+shift+s');
    expect(getShortcut('save')).toBe('mod+shift+s');
    // setting back to the default removes the override
    setShortcut('save', 'mod+s');
    expect(getShortcut('save')).toBe('mod+s');
    expect(window.localStorage.getItem('mars.settings.shortcuts')).toBe('{}');
  });

  it('ignores corrupt stored overrides', () => {
    window.localStorage.setItem('mars.settings.shortcuts', '{bad');
    expect(getShortcut('save')).toBe('mod+s');
  });

  it('resetShortcuts clears stored overrides', () => {
    setShortcut('save', 'f9');
    expect(getShortcut('save')).toBe('f9');
    resetShortcuts();
    expect(getShortcut('save')).toBe('mod+s');
  });
});
