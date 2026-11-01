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
  { id: 'duplicate', label: 'Duplicate cell', defaultCombo: 'shift+c' },
  { id: 'cut-cell', label: 'Cut cell', defaultCombo: 'x' },
  { id: 'copy-cell', label: 'Copy cell', defaultCombo: 'c' },
  { id: 'paste-below', label: 'Paste cell below', defaultCombo: 'v' },
  { id: 'paste-above', label: 'Paste cell above', defaultCombo: 'shift+v' },
  { id: 'interrupt', label: 'Interrupt kernel (press twice)', defaultCombo: 'i' },
  { id: 'toggle-outputs', label: 'Toggle cell outputs', defaultCombo: 'o' },
  { id: 'undo-cell', label: 'Undo', defaultCombo: 'z' },
  { id: 'select-up', label: 'Select cell above', defaultCombo: 'arrowup' },
  { id: 'select-down', label: 'Select cell below', defaultCombo: 'arrowdown' },
  { id: 'help', label: 'Keyboard shortcuts', defaultCombo: 'h' },
  { id: 'heading-1', label: 'Markdown heading 1', defaultCombo: '1' },
  { id: 'heading-2', label: 'Markdown heading 2', defaultCombo: '2' },
  { id: 'heading-3', label: 'Markdown heading 3', defaultCombo: '3' },
  { id: 'heading-4', label: 'Markdown heading 4', defaultCombo: '4' },
  { id: 'heading-5', label: 'Markdown heading 5', defaultCombo: '5' },
  { id: 'heading-6', label: 'Markdown heading 6', defaultCombo: '6' },
  { id: 'toggle-line-numbers', label: 'Toggle line numbers', defaultCombo: 'shift+l' },
  { id: 'shortcuts-help', label: 'Keyboard shortcuts', defaultCombo: '?' },
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
