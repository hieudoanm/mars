import { type FC, useEffect, useState } from 'react';
import { LuMonitor, LuRotateCcw, LuSun, LuX } from 'react-icons/lu';
import { Checkbox, IconButton } from '../shared';
import type { EditorPreferences } from '../../hooks/useEditorPreferences';
import {
  SHORTCUT_ACTIONS,
  comboFromEvent,
  formatCombo,
  getShortcut,
  resetShortcuts,
  setShortcut,
} from '../../lib/shortcuts';

export type ThemePreference = 'mars-dark' | 'mars-light';

const THEME_KEY = 'mars.settings.theme';

export const readThemePreference = (): ThemePreference => {
  try {
    return window.localStorage.getItem(THEME_KEY) === 'mars-light' ? 'mars-light' : 'mars-dark';
  } catch {
    return 'mars-dark';
  }
};

export const applyTheme = (theme: ThemePreference): void => {
  document.documentElement.dataset.theme = theme;
  try {
    window.localStorage.setItem(THEME_KEY, theme);
  } catch {
    // storage unavailable
  }
};

type SettingsDialogProps = {
  open: boolean;
  theme: ThemePreference;
  onThemeChange: (theme: ThemePreference) => void;
  editorPreferences: EditorPreferences;
  onEditorPreferencesChange: (preferences: EditorPreferences) => void;
  onClose: () => void;
};

const ShortcutRow: FC<{ id: string; label: string; defaultCombo: string }> = ({
  id,
  label,
  defaultCombo,
}) => {
  const [recording, setRecording] = useState(false);
  const [combo, setCombo] = useState(() => getShortcut(id));

  useEffect(() => {
    if (!recording) return;
    const onKey = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();
      if (event.key === 'Escape') {
        setRecording(false);
        return;
      }
      const next = comboFromEvent(event);
      setShortcut(id, next);
      setCombo(next);
      setRecording(false);
    };
    window.addEventListener('keydown', onKey, true);
    return () => window.removeEventListener('keydown', onKey, true);
  }, [recording, id]);

  return (
    <li className='flex items-center justify-between gap-2 text-sm'>
      <span className='text-neutral-300'>{label}</span>
      <button
        type='button'
        aria-label={`Rebind ${label}`}
        className={`rounded border px-1.5 py-0.5 font-mono text-xs ${recording ? 'border-orange-600 bg-orange-950 text-orange-200' : 'border-neutral-700 bg-neutral-950 text-neutral-400 hover:border-neutral-500'}`}
        onClick={() => setRecording(true)}>
        {recording ? 'Press keys…' : formatCombo(combo)}
      </button>
      {combo !== defaultCombo ? (
        <button
          type='button'
          aria-label={`Reset ${label}`}
          className='text-neutral-600 hover:text-neutral-300'
          onClick={() => {
            setShortcut(id, defaultCombo);
            setCombo(defaultCombo);
          }}>
          <LuRotateCcw aria-hidden='true' className='h-3 w-3' />
        </button>
      ) : null}
    </li>
  );
};

const SettingsDialog: FC<SettingsDialogProps> = ({
  open,
  theme,
  onThemeChange,
  editorPreferences,
  onEditorPreferencesChange,
  onClose,
}) => {
  const [shortcutsVersion, setShortcutsVersion] = useState(0);

  useEffect(() => {
    if (!open) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [open, onClose]);

  if (!open) return null;
  return (
    <div
      className='fixed inset-0 z-50 flex items-center justify-center bg-black/60'
      role='dialog'
      aria-modal='true'
      aria-label='Settings'>
      <div className='max-h-[85vh] w-full max-w-md overflow-y-auto rounded-xl border border-neutral-700 bg-neutral-900 p-4 shadow-2xl'>
        <div className='mb-4 flex items-center justify-between'>
          <h2 className='text-sm font-semibold text-neutral-100'>Settings</h2>
          <IconButton
            label='Close'
            icon={<LuX aria-hidden='true' className='h-4 w-4' />}
            onClick={onClose}
          />
        </div>
        <p className='mb-1 text-xs font-medium tracking-wide text-neutral-500 uppercase'>Theme</p>
        <div className='mb-4 flex gap-2'>
          {(['mars-dark', 'mars-light'] as const).map((option) => (
            <button
              key={option === 'mars-dark' ? 'Mars Dark' : 'Mars Light'}
              type='button'
              aria-pressed={theme === option}
              className={`flex flex-1 items-center justify-center gap-2 rounded-md border px-3 py-2 text-sm ${theme === option ? 'border-primary bg-primary/10 text-primary' : 'border-base-300 text-base-content/60 hover:bg-base-200'}`}
              onClick={() => onThemeChange(option)}>
              {option === 'mars-dark' ? (
                <LuMonitor aria-hidden='true' className='h-4 w-4' />
              ) : (
                <LuSun aria-hidden='true' className='h-4 w-4' />
              )}
              {option === 'mars-dark' ? 'Mars Dark' : 'Mars Light'}
            </button>
          ))}
        </div>
        <p className='mb-1 text-xs font-medium tracking-wide text-neutral-500 uppercase'>Editor</p>
        <div className='mb-4 flex flex-col gap-1.5'>
          <Checkbox
            label='Autocomplete'
            checked={editorPreferences.autocomplete}
            onChange={(event) =>
              onEditorPreferencesChange({
                ...editorPreferences,
                autocomplete: event.target.checked,
              })
            }
          />
          <Checkbox
            label='Code folding'
            checked={editorPreferences.folding}
            onChange={(event) =>
              onEditorPreferencesChange({ ...editorPreferences, folding: event.target.checked })
            }
          />
        </div>
        <div className='mb-2 flex items-center justify-between'>
          <p className='text-xs font-medium tracking-wide text-neutral-500 uppercase'>
            Keyboard shortcuts
          </p>
          <button
            type='button'
            className='text-xs text-neutral-500 hover:text-neutral-300'
            onClick={() => {
              resetShortcuts();
              setShortcutsVersion((v) => v + 1);
            }}>
            Reset all
          </button>
        </div>
        <p className='mb-2 text-xs text-neutral-600'>Click a shortcut, then press the new keys.</p>
        <ul key={shortcutsVersion} className='space-y-1'>
          {SHORTCUT_ACTIONS.map((action) => (
            <ShortcutRow key={action.id} {...action} />
          ))}
        </ul>
      </div>
    </div>
  );
};

SettingsDialog.displayName = 'SettingsDialog';
export default SettingsDialog;
