import { type FC, type ReactNode } from 'react';
import {
  LuArrowDownToLine,
  LuArrowUpToLine,
  LuCircleStop,
  LuEraser,
  LuFileCode,
  LuFileText,
  LuListChecks,
  LuPlay,
  LuTrash2,
} from 'react-icons/lu';
import type { EditorPreferences } from '../../hooks/useEditorPreferences';
import { Checkbox, IconButton, TextInput } from '../shared';

export type NotebookToolbarControlsProps = {
  isExecuting: boolean;
  executionTimeout: string;
  fontSize: number;
  preferences: EditorPreferences;
  onTimeoutChange: (value: string) => void;
  onFontSizeChange: (value: number) => void;
  onPreferencesChange: (update: (current: EditorPreferences) => EditorPreferences) => void;
  onCancel: () => void;
  onRunCurrent: () => void;
  onRunAll: () => void;
  onRunAbove: () => void;
  onRunBelow: () => void;
  onClearOutputs: () => void;
  canDeleteCell: boolean;
  onDeleteCell: () => void;
  onAddMarkdown: () => void;
  onAddCode: () => void;
};

type ToolbarGroupProps = {
  label: string;
  children: ReactNode;
};

const iconClassName = 'h-4 w-4';

const ToolbarGroup: FC<ToolbarGroupProps> = ({ label, children }) => (
  <div
    role='group'
    aria-label={label}
    data-component='NotebookToolbarGroup'
    className='flex items-center gap-1 rounded-xl border border-neutral-800 bg-neutral-900/60 p-1'>
    {children}
  </div>
);

const NotebookToolbarControls: FC<NotebookToolbarControlsProps> = ({
  isExecuting,
  executionTimeout,
  fontSize,
  preferences,
  onTimeoutChange,
  onFontSizeChange,
  onPreferencesChange,
  onCancel,
  onRunCurrent,
  onRunAll,
  onRunAbove,
  onRunBelow,
  onClearOutputs,
  canDeleteCell,
  onDeleteCell,
  onAddMarkdown,
  onAddCode,
}) => {
  return (
    <div data-component='NotebookToolbarControls' className='flex flex-wrap items-center gap-3'>
      <TextInput
        label='Limit (ms)'
        type='number'
        min='1'
        placeholder='Unlimited'
        value={executionTimeout}
        onChange={(event) => onTimeoutChange(event.target.value)}
        aria-label='Execution timeout in milliseconds'
        className='w-24'
      />
      <ToolbarGroup label='Execution controls'>
        {isExecuting && (
          <IconButton
            label='Stop'
            variant='danger'
            icon={<LuCircleStop aria-hidden='true' className={iconClassName} />}
            onClick={onCancel}
          />
        )}
        <IconButton
          label='Run current cell'
          disabled={isExecuting}
          icon={<LuPlay aria-hidden='true' className={iconClassName} />}
          onClick={onRunCurrent}
        />
        <IconButton
          label='Run all (F6)'
          disabled={isExecuting}
          icon={<LuListChecks aria-hidden='true' className={iconClassName} />}
          onClick={onRunAll}
        />
        <IconButton
          label='Run above'
          disabled={isExecuting}
          icon={<LuArrowUpToLine aria-hidden='true' className={iconClassName} />}
          onClick={onRunAbove}
        />
        <IconButton
          label='Run below'
          disabled={isExecuting}
          icon={<LuArrowDownToLine aria-hidden='true' className={iconClassName} />}
          onClick={onRunBelow}
        />
      </ToolbarGroup>
      <ToolbarGroup label='Output controls'>
        <IconButton
          label='Clear outputs'
          icon={<LuEraser aria-hidden='true' className={iconClassName} />}
          onClick={onClearOutputs}
        />
      </ToolbarGroup>
      <ToolbarGroup label='Cell controls'>
        <IconButton
          label='Add Markdown'
          icon={<LuFileText aria-hidden='true' className={iconClassName} />}
          onClick={onAddMarkdown}
        />
        <IconButton
          label='Add Code'
          variant='primary'
          icon={<LuFileCode aria-hidden='true' className={iconClassName} />}
          onClick={onAddCode}
        />
        <IconButton
          label='Delete selected cell (D, D)'
          variant='danger'
          disabled={!canDeleteCell}
          icon={<LuTrash2 aria-hidden='true' className={iconClassName} />}
          onClick={onDeleteCell}
        />
      </ToolbarGroup>
      <TextInput
        label='Font'
        type='number'
        min='10'
        max='28'
        value={fontSize}
        aria-label='Editor font size'
        onChange={(event) => onFontSizeChange(Number(event.target.value))}
        className='w-16'
      />
      <Checkbox
        label='Autocomplete'
        checked={preferences.autocomplete}
        onChange={(event) =>
          onPreferencesChange((current) => ({ ...current, autocomplete: event.target.checked }))
        }
      />
      <Checkbox
        label='Folding'
        checked={preferences.folding}
        onChange={(event) =>
          onPreferencesChange((current) => ({ ...current, folding: event.target.checked }))
        }
      />
    </div>
  );
};

NotebookToolbarControls.displayName = 'NotebookToolbarControls';

export default NotebookToolbarControls;
