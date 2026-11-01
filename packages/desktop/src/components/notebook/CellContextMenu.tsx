import { type CSSProperties, type FC, useEffect, useRef } from 'react';
import { cn } from '../shared/cn';

export type ContextMenuItem = {
  label: string;
  onSelect: () => void;
  disabled?: boolean;
  danger?: boolean;
  separatorBefore?: boolean;
};

type CellContextMenuProps = {
  x: number;
  y: number;
  items: ContextMenuItem[];
  onClose: () => void;
};

const CellContextMenu: FC<CellContextMenuProps> = ({ x, y, items, onClose }) => {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const onPointerDown = (event: PointerEvent) => {
      if (ref.current && !ref.current.contains(event.target as Node)) onClose();
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose();
    };
    window.addEventListener('pointerdown', onPointerDown, true);
    window.addEventListener('keydown', onKeyDown, true);
    window.addEventListener('blur', onClose);
    return () => {
      window.removeEventListener('pointerdown', onPointerDown, true);
      window.removeEventListener('keydown', onKeyDown, true);
      window.removeEventListener('blur', onClose);
    };
  }, [onClose]);

  // Keep the menu inside the viewport.
  const style: CSSProperties = {
    left: Math.min(x, window.innerWidth - 220),
    top: Math.min(y, window.innerHeight - items.length * 32 - 16),
  };

  return (
    <div
      ref={ref}
      role='menu'
      data-component='CellContextMenu'
      className='fixed z-50 min-w-44 overflow-hidden rounded-lg border border-neutral-700 bg-neutral-900 py-1 shadow-xl'
      style={style}>
      {items.map((item, index) => (
        <div key={`${item.label}-${index}`}>
          {item.separatorBefore && index > 0 ? (
            <div className='my-1 border-t border-neutral-800' />
          ) : null}
          <button
            type='button'
            role='menuitem'
            disabled={item.disabled}
            className={cn(
              'block w-full px-3 py-1.5 text-left text-sm',
              item.danger
                ? 'text-red-400 hover:bg-red-950/60'
                : 'text-neutral-200 hover:bg-neutral-800',
              item.disabled && 'cursor-not-allowed opacity-40 hover:bg-transparent',
            )}
            onClick={() => {
              onClose();
              item.onSelect();
            }}>
            {item.label}
          </button>
        </div>
      ))}
    </div>
  );
};

CellContextMenu.displayName = 'CellContextMenu';
export default CellContextMenu;
