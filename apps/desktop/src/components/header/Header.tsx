import { type FC } from 'react';
import HeaderActions, { type HeaderActionsProps } from './HeaderActions';
import HeaderBrand from './HeaderBrand';

export type HeaderProps = HeaderActionsProps;

const Header: FC<HeaderProps> = ({
  status,
  mode,
  onOpen,
  onSave,
  onExport,
  onImportIpynb,
  onExportIpynb,
  onRestart,
  isExecuting,
  sidebarOpen,
  onToggleSidebar,
  explorerOpen,
  onToggleExplorer,
}) => {
  return (
    <header
      data-component='Header'
      className='flex min-h-16 items-center justify-between border-b border-neutral-800 bg-neutral-900 px-5 py-3'>
      <HeaderBrand />
      <HeaderActions
        status={status}
        mode={mode}
        onOpen={onOpen}
        onSave={onSave}
        onExport={onExport}
        onImportIpynb={onImportIpynb}
        onExportIpynb={onExportIpynb}
        onRestart={onRestart}
        isExecuting={isExecuting}
        sidebarOpen={sidebarOpen}
        onToggleSidebar={onToggleSidebar}
        explorerOpen={explorerOpen}
        onToggleExplorer={onToggleExplorer}
      />
    </header>
  );
};

Header.displayName = 'Header';

export default Header;
