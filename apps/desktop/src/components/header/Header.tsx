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
  onRestart,
  isExecuting,
}) => {
  return (
    <header
      data-component='Header'
      className='flex min-h-20 items-center justify-between border-b border-neutral-800 bg-neutral-900 px-7 py-4'>
      <HeaderBrand />
      <HeaderActions
        status={status}
        mode={mode}
        onOpen={onOpen}
        onSave={onSave}
        onExport={onExport}
        onRestart={onRestart}
        isExecuting={isExecuting}
      />
    </header>
  );
};

Header.displayName = 'Header';

export default Header;
