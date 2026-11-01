import { type FC, type ReactNode } from 'react';
import {
  LuFileCode2,
  LuFileDown,
  LuFileInput,
  LuFileOutput,
  LuFileText,
  LuFolderOpen,
  LuPanelRight,
  LuPlay,
  LuRotateCcw,
  LuSave,
  LuTerminal,
} from 'react-icons/lu';
import { Badge, IconButton } from '../shared';

export type NotebookHeaderMode = 'command' | 'edit';

export type NotebookHeaderActionsProps = {
  status: string;
  mode: NotebookHeaderMode;
  onOpen: () => void;
  onSave: () => void;
  onExport: () => void;
  onExportHtml: () => void;
  onExportPdf: () => void;
  onImportIpynb: () => void;
  onExportIpynb: () => void;
  onRestart: () => void;
  onRestartRunAll: () => void;
  onOpenTerminal?: () => void;
  isExecuting?: boolean;
  sidebarOpen: boolean;
  onToggleSidebar: () => void;
};

const iconClassName = 'h-3.5 w-3.5';

type ActionGroupProps = {
  label: string;
  children: ReactNode;
};

const ActionGroup: FC<ActionGroupProps> = ({ label, children }) => (
  <div
    role='group'
    aria-label={label}
    data-component='NotebookHeaderActionGroup'
    className='flex items-center gap-1 border-l border-neutral-800 pl-3'>
    {children}
  </div>
);

const NotebookHeaderActions: FC<NotebookHeaderActionsProps> = ({
  status,
  mode,
  onOpen,
  onSave,
  onExport,
  onExportHtml,
  onExportPdf,
  onImportIpynb,
  onExportIpynb,
  onRestart,
  onRestartRunAll,
  onOpenTerminal,
  isExecuting = false,
  sidebarOpen,
  onToggleSidebar,
}) => {
  return (
    <div data-component='NotebookHeaderActions' className='flex items-center gap-4'>
      <Badge dot dotTone='success' dotClassName='shadow-sm'>
        {status} · {mode === 'command' ? 'Command mode' : 'Edit mode'}
      </Badge>
      <ActionGroup label='File actions'>
        <IconButton
          label='Open'
          icon={<LuFolderOpen aria-hidden='true' className={iconClassName} />}
          onClick={onOpen}
        />
        <IconButton
          label='Save'
          variant='primary'
          icon={<LuSave aria-hidden='true' className={iconClassName} />}
          onClick={onSave}
        />
      </ActionGroup>
      <ActionGroup label='Import and export'>
        <IconButton
          label='Export Markdown'
          icon={<LuFileText aria-hidden='true' className={iconClassName} />}
          onClick={onExport}
        />
        <IconButton
          label='Export HTML'
          icon={<LuFileCode2 aria-hidden='true' className={iconClassName} />}
          onClick={onExportHtml}
        />
        <IconButton
          label='Export PDF'
          icon={<LuFileDown aria-hidden='true' className={iconClassName} />}
          onClick={onExportPdf}
        />
        <IconButton
          label='Import .ipynb'
          icon={<LuFileInput aria-hidden='true' className={iconClassName} />}
          onClick={onImportIpynb}
        />
        <IconButton
          label='Export .ipynb'
          icon={<LuFileOutput aria-hidden='true' className={iconClassName} />}
          onClick={onExportIpynb}
        />
        {onOpenTerminal ? (
          <IconButton
            label='Open containing folder in terminal'
            icon={<LuTerminal aria-hidden='true' className={iconClassName} />}
            onClick={onOpenTerminal}
          />
        ) : null}
      </ActionGroup>
      <ActionGroup label='Kernel actions'>
        <IconButton
          label='Restart runtime and clear outputs'
          disabled={isExecuting}
          icon={<LuRotateCcw aria-hidden='true' className={iconClassName} />}
          onClick={onRestart}
        />
        <IconButton
          label='Restart runtime and run all'
          disabled={isExecuting}
          icon={<LuPlay aria-hidden='true' className={iconClassName} />}
          onClick={onRestartRunAll}
        />
      </ActionGroup>
      <span aria-hidden='true' className='h-6 w-px bg-neutral-800' />
      <IconButton
        label={sidebarOpen ? 'Hide notebook sidebar' : 'Show notebook sidebar'}
        icon={<LuPanelRight aria-hidden='true' className={iconClassName} />}
        onClick={onToggleSidebar}
      />
    </div>
  );
};

NotebookHeaderActions.displayName = 'NotebookHeaderActions';

export default NotebookHeaderActions;
