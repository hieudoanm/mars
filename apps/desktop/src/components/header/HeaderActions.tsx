import { type FC } from 'react';
import {
  LuDownload,
  LuFileInput,
  LuFileOutput,
  LuFolderOpen,
  LuRotateCcw,
  LuSave,
} from 'react-icons/lu';
import { Badge, IconButton } from '../shared';

export type HeaderMode = 'command' | 'edit';

export type HeaderActionsProps = {
  status: string;
  mode: HeaderMode;
  onOpen: () => void;
  onSave: () => void;
  onExport: () => void;
  onImportIpynb: () => void;
  onExportIpynb: () => void;
  onRestart: () => void;
  isExecuting?: boolean;
};

const iconClassName = 'h-4 w-4';

const HeaderActions: FC<HeaderActionsProps> = ({
  status,
  mode,
  onOpen,
  onSave,
  onExport,
  onImportIpynb,
  onExportIpynb,
  onRestart,
  isExecuting = false,
}) => {
  return (
    <div data-component='HeaderActions' className='flex items-center gap-3'>
      <Badge dot dotTone='success' dotClassName='shadow-sm'>
        {status} · {mode === 'command' ? 'Command mode' : 'Edit mode'}
      </Badge>
      <div className='flex items-center gap-2'>
        <IconButton
          label='Open'
          icon={<LuFolderOpen aria-hidden='true' className={iconClassName} />}
          onClick={onOpen}
        />
        <IconButton
          label='Save'
          icon={<LuSave aria-hidden='true' className={iconClassName} />}
          onClick={onSave}
        />
        <IconButton
          label='Export Markdown'
          icon={<LuDownload aria-hidden='true' className={iconClassName} />}
          onClick={onExport}
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
      </div>
      <span aria-hidden='true' className='h-6 w-px bg-neutral-800' />
      <IconButton
        label='Restart runtime'
        disabled={isExecuting}
        icon={<LuRotateCcw aria-hidden='true' className={iconClassName} />}
        onClick={onRestart}
      />
    </div>
  );
};

HeaderActions.displayName = 'HeaderActions';

export default HeaderActions;
