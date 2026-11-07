import { type FC, useMemo, useState } from 'react';
import { LuCaseSensitive, LuReplace, LuRegex, LuSearch, LuWholeWord, LuX } from 'react-icons/lu';
import type { NotebookCell } from '../../types/types';
import { createSearchPattern, matchesPattern, DEFAULT_SEARCH_OPTIONS } from '../../lib/textSearch';
import type { SearchOptions } from '../../lib/textSearch';
import { IconButton } from '../shared';

export type SearchMatch = {
  cellId: string;
  cellIndex: number;
  cellType: string;
  line: string;
  lineNumber: number;
  character: number;
};

type NotebookSearchDialogProps = {
  cells: NotebookCell[];
  open: boolean;
  onClose: () => void;
  onSelectCell: (cellId: string, lineNumber: number, character: number) => void;
  onReplaceAll?: (find: string, replace: string, options: SearchOptions) => void;
};

export const searchNotebookCells = (
  cells: NotebookCell[],
  query: string,
  options: SearchOptions = DEFAULT_SEARCH_OPTIONS,
): SearchMatch[] => {
  const pattern = createSearchPattern(query, options);
  if (!pattern) return [];
  const matches: SearchMatch[] = [];
  cells.forEach((cell, cellIndex) => {
    cell.source.split('\n').forEach((line, lineIndex) => {
      if (matchesPattern(pattern, line)) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        matches.push({
          cellId: cell.id,
          cellIndex,
          cellType: cell.cell_type,
          line: line.trim(),
          lineNumber: lineIndex + 1,
          character: match?.index ?? 0,
        });
      }
    });
  });
  return matches;
};

const toggleClass = (active: boolean): string =>
  active ? 'text-orange-400' : 'text-neutral-500 hover:text-neutral-300';

const NotebookSearchDialog: FC<NotebookSearchDialogProps> = ({
  cells,
  open,
  onClose,
  onSelectCell,
  onReplaceAll,
}) => {
  const [query, setQuery] = useState('');
  const [replaceText, setReplaceText] = useState('');
  const [options, setOptions] = useState<SearchOptions>(DEFAULT_SEARCH_OPTIONS);
  const [confirmReplace, setConfirmReplace] = useState(false);
  const matches = useMemo(
    () => searchNotebookCells(cells, query, options),
    [cells, query, options],
  );
  if (!open) return null;
  const toggle = (key: keyof SearchOptions) =>
    setOptions((current) => ({ ...current, [key]: !current[key] }));
  return (
    <div
      className='fixed inset-0 z-50 flex items-start justify-center bg-black/60 pt-24'
      role='dialog'
      aria-modal='true'
      aria-label='Search notebook'>
      <div className='w-full max-w-xl rounded-xl border border-neutral-700 bg-neutral-900 shadow-2xl'>
        <div className='flex items-center gap-1 border-b border-neutral-800 px-3 py-2'>
          <LuSearch aria-hidden='true' className='h-4 w-4 shrink-0 text-neutral-500' />
          <input
            autoFocus
            value={query}
            aria-label='Search query'
            placeholder='Search all cells…'
            className='min-w-0 flex-1 bg-transparent text-sm text-neutral-100 outline-none placeholder:text-neutral-600'
            onChange={(event) => setQuery(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === 'Escape') onClose();
              if (event.key === 'Enter' && matches[0]) {
                onSelectCell(matches[0].cellId, matches[0].lineNumber, matches[0].character);
                onClose();
              }
            }}
          />
          <button
            type='button'
            aria-label='Match case'
            aria-pressed={options.caseSensitive}
            title='Match case'
            className={`rounded p-1 ${toggleClass(options.caseSensitive)}`}
            onClick={() => toggle('caseSensitive')}>
            <LuCaseSensitive aria-hidden='true' className='h-4 w-4' />
          </button>
          <button
            type='button'
            aria-label='Match whole word'
            aria-pressed={options.wholeWord}
            title='Match whole word'
            className={`rounded p-1 ${toggleClass(options.wholeWord)}`}
            onClick={() => toggle('wholeWord')}>
            <LuWholeWord aria-hidden='true' className='h-4 w-4' />
          </button>
          <button
            type='button'
            aria-label='Use regular expression'
            aria-pressed={options.regex}
            title='Use regular expression'
            className={`rounded p-1 ${toggleClass(options.regex)}`}
            onClick={() => toggle('regex')}>
            <LuRegex aria-hidden='true' className='h-4 w-4' />
          </button>
          {onReplaceAll ? (
            <>
              <input
                value={replaceText}
                aria-label='Replace with'
                placeholder='Replace with…'
                className='w-36 border-l border-neutral-800 bg-transparent py-1 pl-3 text-sm text-neutral-100 outline-none placeholder:text-neutral-600'
                onChange={(event) => setReplaceText(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === 'Enter' && query.trim()) {
                    onReplaceAll(query, replaceText, options);
                  }
                  if (event.key === 'Escape') onClose();
                }}
              />
              <IconButton
                label='Replace all occurrences'
                disabled={!query.trim() || matches.length === 0}
                icon={<LuReplace aria-hidden='true' className='h-4 w-4' />}
                onClick={() => setConfirmReplace(true)}
              />
            </>
          ) : null}
          <IconButton
            label='Close search'
            icon={<LuX aria-hidden='true' className='h-4 w-4' />}
            onClick={onClose}
          />
        </div>
        {confirmReplace ? (
          <div className='flex items-center justify-between gap-3 border-b border-neutral-800 px-3 py-2 text-sm'>
            <span className='text-neutral-300'>
              Replace {matches.length} occurrence{matches.length === 1 ? '' : 's'}?
            </span>
            <span className='flex items-center gap-2'>
              <button
                type='button'
                className='rounded-md border border-neutral-700 px-2.5 py-1 text-xs text-neutral-300 hover:bg-neutral-800'
                onClick={() => setConfirmReplace(false)}>
                Cancel
              </button>
              <button
                type='button'
                className='rounded-md bg-orange-600 px-2.5 py-1 text-xs font-medium text-white hover:bg-orange-500'
                onClick={() => {
                  setConfirmReplace(false);
                  onReplaceAll?.(query, replaceText, options);
                }}>
                Replace all
              </button>
            </span>
          </div>
        ) : null}
        <div className='max-h-80 overflow-y-auto p-2'>
          {query.trim() && matches.length === 0 ? (
            <p className='px-2 py-3 text-sm text-neutral-500'>
              {options.regex ? 'Invalid pattern or no matches.' : 'No matches.'}
            </p>
          ) : null}
          {matches.slice(0, 100).map((match, index) => (
            <button
              key={`${match.cellId}-${match.lineNumber}-${index}`}
              type='button'
              className='flex w-full items-baseline gap-2 rounded-md px-2 py-1.5 text-left text-sm hover:bg-neutral-800'
              onClick={() => {
                onSelectCell(match.cellId, match.lineNumber, match.character);
                onClose();
              }}>
              <span className='shrink-0 text-xs text-neutral-500'>
                Cell {match.cellIndex + 1} · {match.cellType}
              </span>
              <span className='truncate font-mono text-xs text-neutral-300'>
                {match.line || '(empty line)'}
              </span>
            </button>
          ))}
          {matches.length > 100 ? (
            <p className='px-2 py-2 text-xs text-neutral-500'>
              Showing first 100 of {matches.length} matches.
            </p>
          ) : null}
        </div>
      </div>
    </div>
  );
};

NotebookSearchDialog.displayName = 'NotebookSearchDialog';
export default NotebookSearchDialog;
