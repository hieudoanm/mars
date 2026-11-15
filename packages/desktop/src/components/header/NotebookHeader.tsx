import { type FC } from 'react';
import NotebookHeaderActions, { type NotebookHeaderActionsProps } from './NotebookHeaderActions';
import NotebookHeaderBrand from './NotebookHeaderBrand';

export type NotebookHeaderProps = NotebookHeaderActionsProps & {
  showBrand?: boolean;
};

const NotebookHeader: FC<NotebookHeaderProps> = ({
  showBrand = true,
  status,
  mode,
  onOpen,
  onSave,
  onExport,
  onExportHtml,
  onExportPdf,
  onImportIpynb,
  onExportIpynb,
  onPresent,
  onRestart,
  onRestartRunAll,
  onOpenTerminal,
  isExecuting,
  sidebarOpen,
  onToggleSidebar,
}) => {
  return (
    <header
      data-component='NotebookHeader'
      className='flex min-h-16 items-center justify-between border-b border-neutral-800 bg-neutral-900 px-5 py-3'>
      {showBrand ? <NotebookHeaderBrand /> : <div className='flex-1' />}
      <NotebookHeaderActions
        status={status}
        mode={mode}
        onOpen={onOpen}
        onSave={onSave}
        onExport={onExport}
        onExportHtml={onExportHtml}
        onExportPdf={onExportPdf}
        onImportIpynb={onImportIpynb}
        onExportIpynb={onExportIpynb}
        onPresent={onPresent}
        onRestart={onRestart}
        onRestartRunAll={onRestartRunAll}
        onOpenTerminal={onOpenTerminal}
        isExecuting={isExecuting}
        sidebarOpen={sidebarOpen}
        onToggleSidebar={onToggleSidebar}
      />
    </header>
  );
};

NotebookHeader.displayName = 'NotebookHeader';

export default NotebookHeader;
