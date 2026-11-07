import { type FC } from 'react';
import {
  LuArrowDown,
  LuArrowUp,
  LuChevronDown,
  LuChevronRight,
  LuEraser,
  LuEye,
  LuEyeOff,
  LuGripVertical,
  LuMerge,
  LuPlay,
  LuScissors,
  LuSearch,
  LuTrash2,
} from 'react-icons/lu';
import type { CellType } from '../../types/types';
import { IconButton } from '../shared';
import { TagsEditor } from './TagsEditor';

const iconClassName = 'h-3.5 w-3.5';

export type NotebookCellHeaderProps = {
  cellId: string;
  cellType: CellType;
  cellLanguage?: string;
  isExecuting: boolean;
  runInfo?: { status: 'success' | 'error' | 'interrupted'; durationMs: number } | null;
  isQueued?: boolean;
  collapsed: boolean;
  canDelete: boolean;
  hasOutputs: boolean;
  markdownPreviewVisible: boolean;
  onChangeType?: (cellType: CellType) => void;
  onChangeLanguage?: (language: string) => void;
  onMoveUp?: () => void;
  onMoveDown?: () => void;
  onMergeNext?: () => void;
  onSplitCell?: () => void;
  tags?: string[];
  onUpdateTags?: (tags: string[]) => void;
  onToggleCollapse: () => void;
  onSearch: () => void;
  onToggleMarkdownPreview: () => void;
  onExecute: () => void;
  onClear: () => void;
  onDelete: () => void;
  onStatus: (status: string) => void;
  onDragCellStart?: (id: string) => void;
  onDragCellEnd?: () => void;
};

const stop = (event: { stopPropagation: () => void }) => event.stopPropagation();

const NotebookCellHeader: FC<NotebookCellHeaderProps> = ({
  cellId,
  cellType,
  cellLanguage = 'rust',
  isExecuting,
  runInfo = null,
  isQueued = false,
  collapsed,
  canDelete,
  hasOutputs,
  markdownPreviewVisible,
  onChangeType,
  onChangeLanguage,
  onMoveUp,
  onMoveDown,
  onMergeNext,
  onSplitCell,
  tags = [],
  onUpdateTags,
  onToggleCollapse,
  onSearch,
  onToggleMarkdownPreview,
  onExecute,
  onClear,
  onDelete,
  onStatus,
  onDragCellStart,
  onDragCellEnd,
}) => (
  <div className='mb-1.5 flex items-center justify-between gap-3'>
    <div className='flex items-center gap-2'>
      <span
        draggable
        role='button'
        aria-label='Drag to reorder cell'
        title='Drag to reorder'
        className='cursor-grab text-neutral-600 hover:text-neutral-300 active:cursor-grabbing'
        onDragStart={(event) => {
          event.dataTransfer.effectAllowed = 'move';
          event.dataTransfer.setData('text/plain', cellId);
          onDragCellStart?.(cellId);
        }}
        onDragEnd={onDragCellEnd}>
        <LuGripVertical aria-hidden='true' className={iconClassName} />
      </span>
      <span className='text-xs tracking-widest text-neutral-400 uppercase'>{cellType}</span>
      {tags.map((tag) => (
        <span
          key={tag}
          className='rounded-full border border-neutral-700 px-1.5 py-px text-[10px] text-neutral-500'>
          {tag}
        </span>
      ))}
      {isQueued && !isExecuting ? (
        <span
          data-component='CellQueuedBadge'
          title='Waiting for the current cell to finish'
          className='text-[10px] font-medium text-neutral-500'>
          Queued
        </span>
      ) : null}
      {isExecuting ? (
        <span
          data-component='CellRunningBadge'
          className='animate-pulse text-xs font-medium text-amber-400'>
          Running…
        </span>
      ) : null}
      {!isExecuting && runInfo ? (
        <span
          data-component='CellRunBadge'
          title={`${runInfo.status} in ${runInfo.durationMs} ms`}
          className={`text-[10px] font-medium ${
            runInfo.status === 'success'
              ? 'text-emerald-500'
              : runInfo.status === 'interrupted'
                ? 'text-amber-400'
                : 'text-red-400'
          }`}>
          {runInfo.status === 'success' ? '✓' : runInfo.status === 'interrupted' ? '‖' : '✗'}{' '}
          {runInfo.durationMs}ms
        </span>
      ) : null}
    </div>
    <div className='flex items-center gap-1.5'>
      <select
        aria-label='Cell type'
        className='rounded border border-neutral-800 bg-neutral-950 px-1 py-0.5 text-xs text-neutral-400'
        value={cellType}
        onClick={stop}
        onChange={(event) => onChangeType?.(event.target.value as CellType)}>
        <option value='code'>code</option>
        <option value='markdown'>markdown</option>
        <option value='raw'>raw</option>
      </select>
      {cellType === 'code' && onChangeLanguage ? (
        <select
          aria-label='Cell language'
          className='rounded border border-neutral-800 bg-neutral-950 px-1 py-0.5 text-xs text-neutral-400'
          value={cellLanguage}
          onClick={stop}
          onChange={(event) => onChangeLanguage(event.target.value)}>
          <option value='rust'>rust</option>
          <option value='text'>text</option>
        </select>
      ) : null}
      {onMoveUp ? (
        <IconButton
          label='Move cell up'
          icon={<LuArrowUp aria-hidden='true' className={iconClassName} />}
          onClick={(event) => {
            stop(event);
            onMoveUp();
          }}
        />
      ) : null}
      {onMoveDown ? (
        <IconButton
          label='Move cell down'
          icon={<LuArrowDown aria-hidden='true' className={iconClassName} />}
          onClick={(event) => {
            stop(event);
            onMoveDown();
          }}
        />
      ) : null}
      {onMergeNext ? (
        <IconButton
          label='Merge with next cell'
          icon={<LuMerge aria-hidden='true' className={iconClassName} />}
          onClick={(event) => {
            stop(event);
            onMergeNext();
          }}
        />
      ) : null}
      {onSplitCell ? (
        <IconButton
          label='Split cell at cursor'
          icon={<LuScissors aria-hidden='true' className={iconClassName} />}
          onClick={(event) => {
            stop(event);
            onSplitCell();
          }}
        />
      ) : null}
      {onUpdateTags ? (
        <TagsEditor tags={tags} onUpdate={onUpdateTags} iconClassName={iconClassName} />
      ) : null}
      <IconButton
        label={collapsed ? 'Expand cell' : 'Collapse cell'}
        icon={
          collapsed ? (
            <LuChevronRight aria-hidden='true' className={iconClassName} />
          ) : (
            <LuChevronDown aria-hidden='true' className={iconClassName} />
          )
        }
        onClick={(event) => {
          stop(event);
          onToggleCollapse();
        }}
      />
      <IconButton
        label='Search (Cmd+F)'
        icon={<LuSearch aria-hidden='true' className={iconClassName} />}
        onClick={(event) => {
          stop(event);
          onSearch();
        }}
      />
      {cellType === 'markdown' && (
        <IconButton
          label={markdownPreviewVisible ? 'Hide Markdown preview' : 'Show Markdown preview'}
          icon={
            markdownPreviewVisible ? (
              <LuEyeOff aria-hidden='true' className={iconClassName} />
            ) : (
              <LuEye aria-hidden='true' className={iconClassName} />
            )
          }
          onClick={(event) => {
            stop(event);
            onToggleMarkdownPreview();
          }}
        />
      )}
      {cellType === 'code' && (
        <>
          <IconButton
            label='Run cell (Shift+Enter)'
            icon={<LuPlay aria-hidden='true' className={iconClassName} />}
            onClick={(event) => {
              stop(event);
              onExecute();
            }}
          />
          {hasOutputs && (
            <IconButton
              label='Clear outputs'
              icon={<LuEraser aria-hidden='true' className={iconClassName} />}
              onClick={(event) => {
                stop(event);
                onClear();
                onStatus('Outputs cleared');
              }}
            />
          )}
        </>
      )}
      <IconButton
        label='Delete cell (D, D)'
        variant='danger'
        disabled={!canDelete}
        icon={<LuTrash2 aria-hidden='true' className={iconClassName} />}
        onClick={(event) => {
          stop(event);
          onDelete();
        }}
      />
    </div>
  </div>
);

NotebookCellHeader.displayName = 'NotebookCellHeader';
export default NotebookCellHeader;
