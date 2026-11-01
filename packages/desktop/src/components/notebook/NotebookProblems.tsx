import { type FC, useEffect, useState } from 'react';
import { LuTriangleAlert } from 'react-icons/lu';
import { Badge } from '../shared';
import { SidebarSection } from './SidebarSection';
import type { NotebookProblem } from './NotebookSidebar';

type NotebookProblemsProps = {
  problems: NotebookProblem[];
  onSelectProblem?: (problem: NotebookProblem) => void;
  onSelectCell: (id: string) => void;
};

const selectProblem =
  (
    onSelectProblem: ((p: NotebookProblem) => void) | undefined,
    onSelectCell: (id: string) => void,
  ) =>
  (problem: NotebookProblem) =>
    (onSelectProblem ?? ((p: NotebookProblem) => onSelectCell(p.cellId)))(problem);

export const NotebookProblems: FC<NotebookProblemsProps> = ({
  problems,
  onSelectProblem,
  onSelectCell,
}) => {
  const [problemFilter, setProblemFilter] = useState<'all' | 'error' | 'warning'>('all');
  const [copiedProblems, setCopiedProblems] = useState(false);
  const [problemCursor, setProblemCursor] = useState(-1);
  useEffect(() => setProblemCursor(-1), [problems, problemFilter]);
  const filteredProblems = problems.filter((problem) =>
    problemFilter === 'all' ? true : problem.severity === problemFilter,
  );
  const handleSelect = selectProblem(onSelectProblem, onSelectCell);
  const step = (delta: number) => {
    const next = (problemCursor + delta + filteredProblems.length) % filteredProblems.length;
    setProblemCursor(next);
    handleSelect(filteredProblems[next]);
  };

  return (
    <SidebarSection
      id='problems'
      title={
        <span className='flex items-center gap-1'>
          <LuTriangleAlert aria-hidden='true' className='h-3 w-3 text-amber-400' />
          Problems
          {filteredProblems.length > 0 ? (
            <Badge tone='warning'>{filteredProblems.length}</Badge>
          ) : null}
        </span>
      }>
      <div className='mb-1 flex items-center gap-1 px-2'>
        {(['all', 'error', 'warning'] as const).map((filter) => (
          <button
            key={filter}
            type='button'
            aria-pressed={problemFilter === filter}
            className={`rounded px-1.5 py-0.5 text-[10px] capitalize ${problemFilter === filter ? 'bg-neutral-700 text-neutral-200' : 'text-neutral-500 hover:text-neutral-300'}`}
            onClick={() => setProblemFilter(filter)}>
            {filter}
          </button>
        ))}
        {filteredProblems.length > 1 ? (
          <span className='ml-auto flex items-center gap-0.5'>
            <button
              type='button'
              aria-label='Previous problem'
              className='rounded px-1 text-[10px] text-neutral-500 hover:text-neutral-300'
              onClick={() => step(-1)}>
              ‹
            </button>
            <button
              type='button'
              aria-label='Next problem'
              className='rounded px-1 text-[10px] text-neutral-500 hover:text-neutral-300'
              onClick={() => step(1)}>
              ›
            </button>
          </span>
        ) : null}
        {filteredProblems.length > 0 ? (
          <button
            type='button'
            className={`${filteredProblems.length > 1 ? '' : 'ml-auto'}text-[10px] text-neutral-500 hover:text-neutral-300`}
            onClick={() => {
              const text = filteredProblems
                .map((problem) => `Cell ${problem.cellIndex + 1}: ${problem.message}`)
                .join('\n');
              void navigator.clipboard
                .writeText(text)
                .then(() => {
                  setCopiedProblems(true);
                  window.setTimeout(() => setCopiedProblems(false), 1500);
                })
                .catch(() => {});
            }}>
            {copiedProblems ? 'Copied' : 'Copy all'}
          </button>
        ) : null}
      </div>
      {filteredProblems.length === 0 ? (
        <p className='px-2 py-1 text-xs text-neutral-600'>No problems</p>
      ) : (
        <ul className='space-y-0.5' aria-label='Notebook problems'>
          {filteredProblems.slice(0, 50).map((problem, index) => (
            <li key={`${problem.cellId}-${index}`}>
              <button
                type='button'
                className='flex w-full items-baseline gap-1.5 rounded-md px-2 py-1 text-left text-xs hover:bg-neutral-800'
                onClick={() => handleSelect(problem)}>
                <span
                  aria-hidden='true'
                  className={`mt-0.5 h-1.5 w-1.5 shrink-0 rounded-full ${problem.severity === 'warning' ? 'bg-amber-400' : 'bg-red-400'}`}
                />
                <span className='shrink-0 text-neutral-500'>
                  Cell {problem.cellIndex + 1}
                  <span className='text-neutral-600'>:{problem.line}</span>
                </span>
                <span className='truncate text-neutral-400'>{problem.message}</span>
              </button>
            </li>
          ))}
        </ul>
      )}
    </SidebarSection>
  );
};

NotebookProblems.displayName = 'NotebookProblems';
