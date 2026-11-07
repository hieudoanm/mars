import type { FC, ReactNode } from 'react';

type MobileDrawerProps = Readonly<{
  open: boolean;
  onClose: () => void;
  children: ReactNode;
}>;

const MobileDrawer: FC<MobileDrawerProps> = ({ open, onClose, children }) => (
  <>
    <div
      className={`fixed inset-0 z-30 bg-black/40 transition-opacity duration-300 md-nav:hidden ${
        open ? 'opacity-100' : 'pointer-events-none opacity-0'
      }`}
      aria-hidden='true'
      onClick={onClose}
    />
    <aside
      id='mobile-navigation'
      className={`fixed left-0 top-0 z-40 h-full w-72 max-w-[80vw] overflow-y-auto bg-white p-6 shadow-xl transition-transform duration-300 md-nav:hidden dark:bg-zinc-900 ${
        open ? 'translate-x-0' : '-translate-x-full'
      }`}
      aria-label='Mobile navigation'
      aria-hidden={!open}>
      <nav className='flex flex-col gap-4 text-sm-plus'>{children}</nav>
    </aside>
  </>
);

export default MobileDrawer;
