import type { FC, ReactNode } from 'react';

type MobileDrawerProps = Readonly<{
  open: boolean;
  onClose: () => void;
  children: ReactNode;
}>;

const MobileDrawer: FC<MobileDrawerProps> = ({ open, onClose, children }) => (
  <>
    <div
      className={`md-nav:hidden fixed inset-0 z-30 bg-black/40 transition-opacity duration-300 ${
        open ? 'opacity-100' : 'pointer-events-none opacity-0'
      }`}
      aria-hidden='true'
      onClick={onClose}
    />
    <aside
      id='mobile-navigation'
      className={`md-nav:hidden fixed top-0 left-0 z-40 h-full w-72 max-w-[80vw] overflow-y-auto bg-white p-6 shadow-xl transition-transform duration-300 dark:bg-zinc-900 ${
        open ? 'translate-x-0' : '-translate-x-full'
      }`}
      aria-label='Mobile navigation'
      aria-hidden={!open}>
      <nav className='text-sm-plus flex flex-col gap-4'>{children}</nav>
    </aside>
  </>
);

export default MobileDrawer;
