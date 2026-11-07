import type { FC, ReactNode } from 'react';

type MobileDrawerProps = Readonly<{
  open: boolean;
  onClose: () => void;
  children: ReactNode;
}>;

const MobileDrawer: FC<MobileDrawerProps> = ({ open, onClose, children }) => {
  if (!open) return null;
  return (
    <>
      <div
        className='md-nav:hidden fixed inset-0 z-30 bg-black/40'
        aria-hidden='true'
        onClick={onClose}
      />
      <aside
        id='mobile-navigation'
        className='md-nav:hidden fixed top-0 left-0 z-40 h-full w-72 max-w-[80vw] overflow-y-auto bg-white p-6 shadow-xl dark:bg-zinc-900'
        aria-label='Mobile navigation'>
        <nav className='text-sm-plus flex flex-col gap-4'>{children}</nav>
      </aside>
    </>
  );
};

export default MobileDrawer;
