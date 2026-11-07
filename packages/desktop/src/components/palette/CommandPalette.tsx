import { type FC, useEffect, useMemo, useRef, useState } from 'react';
import { LuCommand } from 'react-icons/lu';

export type PaletteCommand = {
  id: string;
  title: string;
  hint?: string;
  run: () => void;
};

type CommandPaletteProps = {
  open: boolean;
  commands: PaletteCommand[];
  onClose: () => void;
};

const CommandPalette: FC<CommandPaletteProps> = ({ open, commands, onClose }) => {
  const [query, setQuery] = useState('');
  const [index, setIndex] = useState(0);
  const listRef = useRef<HTMLDivElement>(null);

  const filtered = useMemo(() => {
    const needle = query.trim().toLowerCase();
    if (!needle) return commands;
    return commands.filter(
      (command) =>
        command.title.toLowerCase().includes(needle) ||
        command.hint?.toLowerCase().includes(needle),
    );
  }, [commands, query]);

  useEffect(() => {
    if (open) {
      setQuery('');
      setIndex(0);
    }
  }, [open]);

  useEffect(() => setIndex(0), [query]);

  useEffect(() => {
    if (!open) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose();
      if (event.key === 'ArrowDown') {
        event.preventDefault();
        setIndex((current) => Math.min(current + 1, filtered.length - 1));
      }
      if (event.key === 'ArrowUp') {
        event.preventDefault();
        setIndex((current) => Math.max(current - 1, 0));
      }
      if (event.key === 'Enter' && filtered[index]) {
        event.preventDefault();
        filtered[index].run();
        onClose();
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [open, filtered, index, onClose]);

  useEffect(() => {
    listRef.current?.querySelector(`[data-index="${index}"]`)?.scrollIntoView({ block: 'nearest' });
  }, [index]);

  if (!open) return null;
  return (
    <div
      className='fixed inset-0 z-50 flex items-start justify-center bg-black/60 pt-28'
      role='dialog'
      aria-modal='true'
      aria-label='Command palette'
      onClick={onClose}>
      <div
        className='w-full max-w-lg overflow-hidden rounded-xl border border-neutral-700 bg-neutral-900 shadow-2xl'
        onClick={(event) => event.stopPropagation()}>
        <div className='flex items-center gap-2 border-b border-neutral-800 px-3 py-2'>
          <LuCommand aria-hidden='true' className='h-4 w-4 text-neutral-500' />
          <input
            autoFocus
            value={query}
            aria-label='Command search'
            placeholder='Type a command…'
            className='min-w-0 flex-1 bg-transparent text-sm text-neutral-100 outline-none placeholder:text-neutral-600'
            onChange={(event) => setQuery(event.target.value)}
          />
        </div>
        <div ref={listRef} className='max-h-80 overflow-y-auto p-1.5'>
          {filtered.length === 0 ? (
            <p className='px-2 py-3 text-sm text-neutral-500'>No matching commands.</p>
          ) : (
            filtered.map((command, commandIndex) => (
              <button
                key={command.id}
                type='button'
                data-index={commandIndex}
                className={`flex w-full items-center justify-between rounded-md px-2 py-1.5 text-left text-sm ${commandIndex === index ? 'bg-orange-950 text-orange-100' : 'text-neutral-300 hover:bg-neutral-800'}`}
                onMouseEnter={() => setIndex(commandIndex)}
                onClick={() => {
                  command.run();
                  onClose();
                }}>
                <span>{command.title}</span>
                {command.hint ? (
                  <span className='text-xs text-neutral-500'>{command.hint}</span>
                ) : null}
              </button>
            ))
          )}
        </div>
      </div>
    </div>
  );
};

CommandPalette.displayName = 'CommandPalette';
export default CommandPalette;
