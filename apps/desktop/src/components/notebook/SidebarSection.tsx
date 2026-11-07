import { type FC, type ReactNode, useState } from 'react';
import { LuChevronDown, LuChevronRight } from 'react-icons/lu';

const STORAGE_KEY = 'mars.sidebar.collapsed';

const readCollapsed = (): ReadonlySet<string> => {
  try {
    const stored = JSON.parse(window.localStorage.getItem(STORAGE_KEY) ?? '[]');
    return new Set(Array.isArray(stored) ? stored.filter((v) => typeof v === 'string') : []);
  } catch {
    return new Set();
  }
};

const writeCollapsed = (collapsed: ReadonlySet<string>) => {
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify([...collapsed]));
  } catch {
    // storage unavailable
  }
};

type SidebarSectionProps = {
  id: string;
  title: ReactNode;
  actions?: ReactNode;
  children: ReactNode;
};

export const SidebarSection: FC<SidebarSectionProps> = ({ id, title, actions, children }) => {
  const [collapsed, setCollapsed] = useState<ReadonlySet<string>>(readCollapsed);
  const open = !collapsed.has(id);
  const toggle = () => {
    setCollapsed((current) => {
      const next = new Set(current);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      writeCollapsed(next);
      return next;
    });
  };
  return (
    <section className='mb-4'>
      <div className='mb-1 flex items-center justify-between gap-1'>
        <button
          type='button'
          aria-expanded={open}
          onClick={toggle}
          className='flex min-w-0 flex-1 items-center gap-1 text-left'>
          {open ? (
            <LuChevronDown aria-hidden='true' className='h-3 w-3 shrink-0 text-neutral-500' />
          ) : (
            <LuChevronRight aria-hidden='true' className='h-3 w-3 shrink-0 text-neutral-500' />
          )}
          <span
            className={`min-w-0 flex-1 text-xs tracking-widest text-neutral-400 uppercase ${open ? '' : ''}`}>
            {title}
          </span>
        </button>
        {actions ? <span className='flex shrink-0 items-center gap-1'>{actions}</span> : null}
      </div>
      {open ? children : null}
    </section>
  );
};

SidebarSection.displayName = 'SidebarSection';
