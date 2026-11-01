import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import SettingsDialog from '../SettingsDialog';
import type { EditorPreferences } from '../../../hooks/useEditorPreferences';

const prefs: EditorPreferences = {
  autocomplete: true,
  folding: true,
  inlayHints: true,
  lineNumbers: true,
};

const renderDialog = (overrides: Partial<EditorPreferences> = {}) => {
  const onChange = vi.fn();
  render(
    <SettingsDialog
      open
      theme='mars-dark'
      onThemeChange={vi.fn()}
      editorPreferences={{ ...prefs, ...overrides }}
      onEditorPreferencesChange={onChange}
      onClose={vi.fn()}
    />,
  );
  return onChange;
};

describe('SettingsDialog editor preferences', () => {
  it('toggles inlay hints', () => {
    const onChange = renderDialog();
    fireEvent.click(screen.getByLabelText('Inlay hints'));
    expect(onChange).toHaveBeenCalledWith({ ...prefs, inlayHints: false });
  });

  it('reflects stored preference state', () => {
    renderDialog({ inlayHints: false });
    expect(screen.getByLabelText('Inlay hints')).not.toBeChecked();
  });
});
