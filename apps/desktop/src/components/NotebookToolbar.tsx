import type { EditorPreferences } from '../hooks/useEditorPreferences';

type NotebookToolbarProps = {
  fileName: string;
  title: string;
  isDirty: boolean;
  isExecuting: boolean;
  executionTimeout: string;
  fontSize: number;
  preferences: EditorPreferences;
  onTimeoutChange: (value: string) => void;
  onFontSizeChange: (value: number) => void;
  onPreferencesChange: (update: (current: EditorPreferences) => EditorPreferences) => void;
  onCancel: () => void;
  onRunAll: () => void;
  onRunAbove: () => void;
  onRunBelow: () => void;
  onClearOutputs: () => void;
  onAddMarkdown: () => void;
  onAddCode: () => void;
};

export default function NotebookToolbar({
  fileName,
  title,
  isDirty,
  isExecuting,
  executionTimeout,
  fontSize,
  preferences,
  onTimeoutChange,
  onFontSizeChange,
  onPreferencesChange,
  onCancel,
  onRunAll,
  onRunAbove,
  onRunBelow,
  onClearOutputs,
  onAddMarkdown,
  onAddCode,
}: NotebookToolbarProps) {
  return (
    <div className='notebook-toolbar'>
      <div>
        <span className='breadcrumb'>
          Workspace / {fileName}
          {isDirty ? ' • unsaved' : ''}
        </span>
        <h2>{title}</h2>
      </div>
      <div className='toolbar-actions'>
        <label className='timeout-control'>
          <span>Limit (ms)</span>
          <input
            type='number'
            min='1'
            placeholder='Unlimited'
            value={executionTimeout}
            onChange={(event) => onTimeoutChange(event.target.value)}
            aria-label='Execution timeout in milliseconds'
          />
        </label>
        {isExecuting && (
          <button className='danger' onClick={onCancel}>
            Stop
          </button>
        )}
        <button onClick={onRunAll}>
          Run all <kbd>F6</kbd>
        </button>
        <button onClick={onRunAbove}>Run above</button>
        <button onClick={onRunBelow}>Run below</button>
        <button onClick={onClearOutputs}>Clear outputs</button>
        <button onClick={onAddMarkdown}>+ Markdown</button>
        <button className='primary' onClick={onAddCode}>
          + Code
        </button>
        <label className='font-size-control'>
          <span>Font</span>
          <input
            type='number'
            min='10'
            max='28'
            value={fontSize}
            aria-label='Editor font size'
            onChange={(event) => onFontSizeChange(Number(event.target.value))}
          />
        </label>
        <label className='editor-preference'>
          <input
            type='checkbox'
            checked={preferences.autocomplete}
            onChange={(event) =>
              onPreferencesChange((current) => ({ ...current, autocomplete: event.target.checked }))
            }
          />
          Autocomplete
        </label>
        <label className='editor-preference'>
          <input
            type='checkbox'
            checked={preferences.folding}
            onChange={(event) =>
              onPreferencesChange((current) => ({ ...current, folding: event.target.checked }))
            }
          />
          Folding
        </label>
      </div>
    </div>
  );
}
