import { type FC, useState } from 'react';
import { LuCopy, LuRefreshCw, LuVariable } from 'react-icons/lu';
import type { KernelVariable } from '../../types/types';
import type { RustSymbol } from '../../lib/rustSymbols';
import { IconButton } from '../shared';
import { SidebarSection } from './SidebarSection';

type NotebookVariablesProps = {
  liveVariables: KernelVariable[];
  symbols: RustSymbol[];
  onRefreshVariables?: () => void;
};

export const NotebookVariables: FC<NotebookVariablesProps> = ({
  liveVariables,
  symbols,
  onRefreshVariables,
}) => {
  const [expandedVariables, setExpandedVariables] = useState<ReadonlySet<string>>(new Set());
  if (liveVariables.length === 0 && symbols.length === 0) return null;
  return (
    <SidebarSection
      id='variables'
      title={
        <span className='flex items-center gap-1'>
          <LuVariable aria-hidden='true' className='h-3 w-3' />{' '}
          {liveVariables.length > 0 ? 'Live variables' : 'Variables & symbols'}
        </span>
      }
      actions={
        onRefreshVariables ? (
          <IconButton
            label='Refresh variables'
            icon={<LuRefreshCw aria-hidden='true' className='h-3 w-3' />}
            onClick={onRefreshVariables}
          />
        ) : undefined
      }>
      {liveVariables.length > 0 ? (
        <ul className='space-y-0.5' aria-label='Live kernel variables'>
          {liveVariables.map((variable) => (
            <li key={variable.name} className='px-2 py-1 text-xs'>
              <div className='flex items-center justify-between gap-2'>
                <span className='truncate font-mono text-neutral-300'>{variable.name}</span>
                <span className='flex shrink-0 items-center gap-1'>
                  <span className='truncate text-neutral-600'>
                    {variable.typeName ?? variable.value ?? ''}
                  </span>
                  <IconButton
                    label={`Copy ${variable.name}`}
                    variant='ghost'
                    className='p-0.5'
                    icon={<LuCopy aria-hidden='true' className='h-3 w-3' />}
                    onClick={() => {
                      void navigator.clipboard
                        ?.writeText(variable.value ?? variable.name)
                        .catch(() => {});
                    }}
                  />
                </span>
              </div>
              {variable.value !== null ? (
                <button
                  type='button'
                  className='block w-full text-left font-mono text-[10px] text-neutral-500 hover:text-neutral-300'
                  title={expandedVariables.has(variable.name) ? 'Collapse value' : 'Expand value'}
                  onClick={() =>
                    setExpandedVariables((current) => {
                      const next = new Set(current);
                      if (next.has(variable.name)) next.delete(variable.name);
                      else next.add(variable.name);
                      return next;
                    })
                  }>
                  <span
                    className={
                      expandedVariables.has(variable.name)
                        ? 'block break-all whitespace-pre-wrap'
                        : 'block truncate'
                    }>
                    {variable.value}
                  </span>
                </button>
              ) : null}
            </li>
          ))}
        </ul>
      ) : (
        <ul className='space-y-0.5' aria-label='Notebook symbols'>
          {symbols.map((symbol) => (
            <li
              key={symbol.name}
              className='flex items-center justify-between gap-2 truncate px-2 py-1 text-xs'>
              <span className='truncate font-mono text-neutral-300'>{symbol.name}</span>
              <span className='shrink-0 text-neutral-600'>{symbol.kind}</span>
            </li>
          ))}
        </ul>
      )}
    </SidebarSection>
  );
};

NotebookVariables.displayName = 'NotebookVariables';
