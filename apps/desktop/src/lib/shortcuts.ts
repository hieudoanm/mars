export type ShortcutAction = {
  id: string;
  label: string;
  defaultCombo: string;
};

export const SHORTCUT_ACTIONS: ShortcutAction[] = [
  { id: 'save', label: 'Save notebook', defaultCombo: 'mod+s' },
  { id: 'undo', label: 'Undo', defaultCombo: 'mod+z' },
  { id: 'redo', label: 'Redo', defaultCombo: 'mod+shift+z' },
  { id: 'redo-alt', label: 'Redo (alternate)', defaultCombo: 'mod+y' },
  { id: 'run-cell', label: 'Run cell and advance', defaultCombo: 'shift+enter' },
  { id: 'run-stay', label: 'Run cell', defaultCombo: 'mod+enter' },
  { id: 'run-insert', label: 'Run cell and insert below', defaultCombo: 'alt+enter' },
  { id: 'run-all', label: 'Run all cells', defaultCombo: 'f6' },
  { id: 'insert-above', label: 'Insert cell above', defaultCombo: 'a' },
  { id: 'insert-below', label: 'Insert cell below', defaultCombo: 'b' },
  { id: 'delete-cell', label: 'Delete cell (press twice)', defaultCombo: 'd' },
  { id: 'to-markdown', label: 'Convert to Markdown', defaultCombo: 'm' },
  { id: 'to-code', label: 'Convert to code', defaultCombo: 'y' },
  { id: 'to-raw', label: 'Convert to raw', defaultCombo: 'r' },
  { id: 'move-down', label: 'Move cell down', defaultCombo: 'j' },
  { id: 'move-up', label: 'Move cell up', defaultCombo: 'k' },
  { id: 'duplicate', label: 'Duplicate cell', defaultCombo: 'c' },
];

const STORAGE_KEY = 'mars.settings.shortcuts';

const readOverrides = (): Record<string, string> => {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return {};
    const parsed: unknown = JSON.parse(raw);
    if (typeof parsed !== 'object' || parsed === null) return {};
    return parsed as Record<string, string>;
  } catch {
    return {};
  }
};

export const getShortcut = (id: string): string => {
  const action = SHORTCUT_ACTIONS.find((candidate) => candidate.id === id);
  if (!action) return '';
  return readOverrides()[id] ?? action.defaultCombo;
};

export const setShortcut = (id: string, combo: string): void => {
  try {
    const overrides = readOverrides();
    const action = SHORTCUT_ACTIONS.find((candidate) => candidate.id === id);
    if (combo === action?.defaultCombo) delete overrides[id];
    else overrides[id] = combo;
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(overrides));
  } catch {
    // storage unavailable
  }
};

export const resetShortcuts = (): void => {
  try {
    window.localStorage.removeItem(STORAGE_KEY);
  } catch {
    // storage unavailable
  }
};

const normalizeKey = (key: string): string => key.toLowerCase();

export const comboFromEvent = (event: KeyboardEvent): string => {
  const parts: string[] = [];
  if (event.metaKey || event.ctrlKey) parts.push('mod');
  if (event.altKey) parts.push('alt');
  if (event.shiftKey) parts.push('shift');
  parts.push(normalizeKey(event.key === ' ' ? 'space' : event.key));
  return parts.join('+');
};

export const comboMatches = (combo: string, event: KeyboardEvent): boolean => {
  const expected = combo.split('+').map(normalizeKey);
  const hasMod = expected.includes('mod');
  const hasAlt = expected.includes('alt');
  const hasShift = expected.includes('shift');
  const key = expected[expected.length - 1];
  const eventMod = event.metaKey || event.ctrlKey;
  if (hasMod !== eventMod) return false;
  if (hasAlt !== event.altKey) return false;
  if (hasShift !== event.shiftKey) return false;
  return normalizeKey(event.key === ' ' ? 'space' : event.key) === key;
};

export const formatCombo = (combo: string): string =>
  combo
    .split('+')
    .map((part) =>
      part === 'mod'
        ? '⌘/Ctrl'
        : part === 'enter'
          ? 'Enter'
          : part === 'escape'
            ? 'Esc'
            : part[0].toUpperCase() + part.slice(1),
    )
    .join(' + ');
