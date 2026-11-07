import { type FC, type ReactNode } from 'react';
import {
  LuArrowDownToLine,
  LuArrowUpToLine,
  LuChevronsDown,
  LuChevronsUp,
  LuCircleStop,
  LuChevronDown,
  LuEraser,
  LuFile,
  LuFileCode,
  LuFileText,
  LuListChecks,
  LuPlay,
  LuTrash2,
} from 'react-icons/lu';
import { IconButton } from '../shared';

export type NotebookToolbarControlsProps = {
  isExecuting: boolean;
  executionTimeout: string;
  fontSize: number;
  onTimeoutChange: (value: string) => void;
  onFontSizeChange: (value: number) => void;
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
  onAddRaw: () => void;
  onCollapseAllCells: () => void;
  onExpandAllCells: () => void;
};

type ToolbarGroupProps = {
  label: string;
  children: ReactNode;
};

const iconClassName = 'h-3.5 w-3.5';

const TIMEOUT_OPTIONS: [string, string][] = [
  ['', 'Unlimited'],
  ['5000', '5 s'],
  ['10000', '10 s'],
  ['30000', '30 s'],
  ['60000', '1 min'],
  ['300000', '5 min'],
  ['900000', '15 min'],
];

const FONT_SIZE_OPTIONS = [10, 11, 12, 13, 14, 16, 18, 20, 24];

const ToolbarGroup: FC<ToolbarGroupProps> = ({ label, children }) => (
  <div
    role='group'
    aria-label={label}
    data-component='NotebookToolbarGroup'
    className='flex items-center gap-0.5 border-l border-neutral-800 pl-3'>
    {children}
  </div>
);

const NotebookToolbarControls: FC<NotebookToolbarControlsProps> = ({
  isExecuting,
  executionTimeout,
  fontSize,
  onTimeoutChange,
  onFontSizeChange,
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
  onAddRaw,
  onCollapseAllCells,
  onExpandAllCells,
}) => {
  return (
    <div
      data-component='NotebookToolbarControls'
      className='flex flex-wrap items-center gap-x-4 gap-y-2'>
      <div className='flex items-center gap-3'>
        <span className='relative flex items-center'>
          <select
            aria-label='Execution timeout in milliseconds'
            title='Execution timeout'
            value={executionTimeout}
            onChange={(event) => onTimeoutChange(event.target.value)}
            className='h-7 w-auto appearance-none rounded-md border border-neutral-700 bg-neutral-900 px-2 pr-5 text-xs text-neutral-200'>
            {TIMEOUT_OPTIONS.map(([value, label]) => (
              <option key={value} value={value}>
                {label}
              </option>
            ))}
            {executionTimeout && !TIMEOUT_OPTIONS.some(([value]) => value === executionTimeout) ? (
              <option value={executionTimeout}>{executionTimeout} ms</option>
            ) : null}
          </select>
          <LuChevronDown
            aria-hidden='true'
            className='pointer-events-none absolute top-1/2 right-1.5 h-3 w-3 -translate-y-1/2 text-neutral-500'
          />
        </span>
        <span className='flex items-center gap-1'>
          <span className='relative flex items-center'>
            <select
              aria-label='Editor font size'
              title='Editor font size'
              value={fontSize}
              onChange={(event) => onFontSizeChange(Number(event.target.value))}
              className='h-7 w-auto appearance-none rounded-md border border-neutral-700 bg-neutral-900 px-2 pr-5 text-xs text-neutral-200'>
              {FONT_SIZE_OPTIONS.map((size) => (
                <option key={size} value={size}>
                  {size}
                </option>
              ))}
              {!FONT_SIZE_OPTIONS.includes(fontSize) ? (
                <option value={fontSize}>{fontSize}</option>
              ) : null}
            </select>
            <LuChevronDown
              aria-hidden='true'
              className='pointer-events-none absolute top-1/2 right-1.5 h-3 w-3 -translate-y-1/2 text-neutral-500'
            />
          </span>
          <span aria-hidden='true' className='text-[10px] text-neutral-600'>
            px
          </span>
        </span>
      </div>
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
      <ToolbarGroup label='View controls'>
        <IconButton
          label='Collapse all cells'
          icon={<LuChevronsDown aria-hidden='true' className={iconClassName} />}
          onClick={onCollapseAllCells}
        />
        <IconButton
          label='Expand all cells'
          icon={<LuChevronsUp aria-hidden='true' className={iconClassName} />}
          onClick={onExpandAllCells}
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
          label='Add Raw'
          icon={<LuFile aria-hidden='true' className={iconClassName} />}
          onClick={onAddRaw}
        />
        <IconButton
          label='Delete selected cell (D, D)'
          variant='danger'
          disabled={!canDeleteCell}
          icon={<LuTrash2 aria-hidden='true' className={iconClassName} />}
          onClick={onDeleteCell}
        />
      </ToolbarGroup>
    </div>
  );
};

NotebookToolbarControls.displayName = 'NotebookToolbarControls';

export default NotebookToolbarControls;
