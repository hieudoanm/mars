import { useState } from 'react';
import type { CellComment } from '../../lib/cellComments';
import { addCellComment, removeCellComment } from '../../lib/cellComments';

type CellCommentsProps = {
  comments: CellComment[];
  onChange: (comments: CellComment[]) => void;
};

const formatDate = (iso: string): string => {
  const date = new Date(iso);
  return Number.isNaN(date.getTime()) ? iso : date.toLocaleString();
};

export const CellComments = ({ comments, onChange }: CellCommentsProps) => {
  const [draft, setDraft] = useState('');
  const [open, setOpen] = useState(comments.length > 0);

  const submit = () => {
    const next = addCellComment(comments, draft);
    if (next !== comments) {
      onChange(next);
      setDraft('');
      setOpen(true);
    }
  };

  return (
    <details
      className='mt-2 rounded-md border border-neutral-800 bg-neutral-950/60'
      open={open}
      onToggle={(event) => setOpen((event.target as HTMLDetailsElement).open)}>
      <summary className='cursor-pointer select-none px-3 py-1.5 text-xs text-neutral-400 hover:text-neutral-200'>
        Comments{comments.length > 0 ? ` (${comments.length})` : ''}
      </summary>
      <div className='border-t border-neutral-800 px-3 py-2'>
        {comments.length > 0 && (
          <ul className='mb-2 space-y-1.5'>
            {comments.map((comment, index) => (
              <li
                key={`${comment.created_at}-${index}`}
                className='flex items-start justify-between gap-2 text-xs'>
                <div className='min-w-0'>
                  <p className='whitespace-pre-wrap text-neutral-300'>{comment.text}</p>
                  <p className='text-[10px] text-neutral-500'>{formatDate(comment.created_at)}</p>
                </div>
                <button
                  type='button'
                  aria-label='Delete comment'
                  className='shrink-0 rounded px-1 text-neutral-500 hover:bg-neutral-800 hover:text-neutral-200'
                  onClick={() => onChange(removeCellComment(comments, index))}>
                  ×
                </button>
              </li>
            ))}
          </ul>
        )}
        <div className='flex gap-2'>
          <input
            type='text'
            value={draft}
            onChange={(event) => setDraft(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === 'Enter') submit();
            }}
            placeholder='Add a comment…'
            aria-label='Add a comment'
            className='min-w-0 flex-1 rounded-md border border-neutral-800 bg-neutral-900 px-2 py-1 text-xs text-neutral-200 placeholder:text-neutral-600 focus:border-orange-700 focus:outline-none'
          />
          <button
            type='button'
            onClick={submit}
            disabled={!draft.trim()}
            className='rounded-md border border-neutral-700 px-2 py-1 text-xs text-neutral-300 hover:bg-neutral-800 disabled:opacity-40'>
            Add
          </button>
        </div>
      </div>
    </details>
  );
};
