import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import { beforeEach } from 'vitest';
import SettingsDialog from '../SettingsDialog';
import { getShortcut } from '../../../lib/shortcuts';
import type { EditorPreferences } from '../../../hooks/useEditorPreferences';

const prefs: EditorPreferences = {
  autocomplete: true,
  folding: true,
  inlayHints: true,
  lineNumbers: true,
};

const renderDialog = (overrides: Partial<EditorPreferences> = {}) => {
  const onChange = vi.fn();
  const onClose = vi.fn();
  const onThemeChange = vi.fn();
  render(
    <SettingsDialog
      open
      theme='mars-dark'
      onThemeChange={onThemeChange}
      editorPreferences={{ ...prefs, ...overrides }}
      onEditorPreferencesChange={onChange}
      onClose={onClose}
    />,
  );
  return { onChange, onClose, onThemeChange };
};

describe('SettingsDialog editor preferences', () => {
  beforeEach(() => window.localStorage.clear());
  it('toggles inlay hints', () => {
    const { onChange } = renderDialog();
    fireEvent.click(screen.getByLabelText('Inlay hints'));
    expect(onChange).toHaveBeenCalledWith({ ...prefs, inlayHints: false });
  });

  it('reflects stored preference state', () => {
    renderDialog({ inlayHints: false });
    expect(screen.getByLabelText('Inlay hints')).not.toBeChecked();
  });

  it('toggles autocomplete, folding, and line numbers', () => {
    const { onChange } = renderDialog();
    fireEvent.click(screen.getByLabelText('Autocomplete'));
    expect(onChange).toHaveBeenLastCalledWith({ ...prefs, autocomplete: false });
    fireEvent.click(screen.getByLabelText('Code folding'));
    expect(onChange).toHaveBeenLastCalledWith({ ...prefs, folding: false });
    fireEvent.click(screen.getByLabelText('Line numbers'));
    expect(onChange).toHaveBeenLastCalledWith({ ...prefs, lineNumbers: false });
  });

  it('switches the color theme', () => {
    const { onThemeChange } = renderDialog();
    fireEvent.click(screen.getByRole('button', { name: /mars light/i }));
    expect(onThemeChange).toHaveBeenCalledWith('mars-light');
  });

  it('closes via the close button and Escape', () => {
    const { onClose } = renderDialog();
    fireEvent.click(screen.getByRole('button', { name: 'Close' }));
    expect(onClose).toHaveBeenCalledTimes(1);
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(onClose).toHaveBeenCalledTimes(2);
  });

  it('renders nothing when closed', () => {
    const { container } = render(
      <SettingsDialog
        open={false}
        theme='mars-dark'
        onThemeChange={vi.fn()}
        editorPreferences={prefs}
        onEditorPreferencesChange={vi.fn()}
        onClose={vi.fn()}
      />,
    );
    expect(container).toBeEmptyDOMElement();
  });

  it('rebinds a shortcut by recording keys', () => {
    renderDialog();
    fireEvent.click(screen.getByRole('button', { name: 'Rebind Save notebook' }));
    expect(screen.getByText('Press keys…')).toBeInTheDocument();
    fireEvent.keyDown(window, { key: 's', metaKey: true, shiftKey: true });
    expect(getShortcut('save')).toBe('mod+shift+s');
  });

  it('cancels shortcut recording with Escape', () => {
    renderDialog();
    fireEvent.click(screen.getByRole('button', { name: 'Rebind Save notebook' }));
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(screen.queryByText('Press keys…')).toBeNull();
    expect(getShortcut('save')).toBe('mod+s');
  });

  it('resets an overridden shortcut and all shortcuts', () => {
    window.localStorage.setItem('mars.settings.shortcuts', JSON.stringify({ save: 'f9' }));
    renderDialog();
    fireEvent.click(screen.getByRole('button', { name: 'Reset Save notebook' }));
    expect(getShortcut('save')).toBe('mod+s');
    window.localStorage.setItem('mars.settings.shortcuts', JSON.stringify({ save: 'f9' }));
    fireEvent.click(screen.getByRole('button', { name: 'Reset all' }));
    expect(getShortcut('save')).toBe('mod+s');
  });
});
