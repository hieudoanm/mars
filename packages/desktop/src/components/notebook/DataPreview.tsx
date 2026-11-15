import { useEffect, useState } from 'react';
import { readTablePreview, type TablePreview } from '../../lib/kernelService';

type DataPreviewProps = {
  path: string;
};

/// Renders a tabular preview for cells carrying `metadata.preview` — a path
/// to a CSV file (absolute or relative to the app working directory).
/// Loads lazily when the section is expanded.
export const DataPreview = ({ path }: DataPreviewProps) => {
  const [open, setOpen] = useState(false);
  const [data, setData] = useState<TablePreview | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!open || data || loading) return;
    setLoading(true);
    readTablePreview(path)
      .then(setData)
      .catch((err: unknown) =>
        setError(err instanceof Error ? err.message : 'Preview failed'),
      )
      .finally(() => setLoading(false));
  }, [open, data, loading, path]);

  return (
    <details
      className='mt-2 rounded-md border border-neutral-800 bg-neutral-950/60'
      onToggle={(event) => setOpen((event.target as HTMLDetailsElement).open)}>
      <summary className='cursor-pointer select-none px-3 py-1.5 text-xs text-neutral-400 hover:text-neutral-200'>
        Data preview{data ? ` (${data.rows.length} rows)` : ''}
      </summary>
      <div className='max-h-72 overflow-auto border-t border-neutral-800'>
        {loading ? (
          <p className='px-3 py-2 text-xs text-neutral-500'>Loading preview…</p>
        ) : error ? (
          <p className='px-3 py-2 text-xs text-red-400'>{error}</p>
        ) : data ? (
          <table className='w-full border-collapse text-left text-xs'>
            <thead className='sticky top-0 bg-neutral-900'>
              <tr>
                {data.columns.map((column) => (
                  <th
                    key={column}
                    className='border-b border-neutral-700 px-3 py-1.5 font-medium text-neutral-300'>
                    {column}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {data.rows.map((row, rowIndex) => (
                <tr key={rowIndex} className='odd:bg-neutral-900/50'>
                  {row.map((value, cellIndex) => (
                    <td
                      key={cellIndex}
                      className='border-b border-neutral-800/60 px-3 py-1 text-neutral-400'>
                      {value ?? ''}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        ) : null}
      </div>
    </details>
  );
};
