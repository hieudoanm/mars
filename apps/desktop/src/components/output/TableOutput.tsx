import { type FC, useMemo, useState } from 'react';
import { LuArrowDown, LuArrowUp, LuCopy, LuSearch } from 'react-icons/lu';
import { OUTPUT_SURFACE } from './outputStyles';

type TableOutputProps = {
  columns: string[];
  rows: string[][];
};

type SortState = { column: number; direction: 'asc' | 'desc' } | null;

const toCsv = (columns: string[], rows: string[][]): string => {
  const escape = (value: string) =>
    /[",\n]/.test(value) ? `"${value.replace(/"/g, '""')}"` : value;
  return [columns, ...rows].map((row) => row.map(escape).join(',')).join('\n');
};

const TableOutput: FC<TableOutputProps> = ({ columns, rows }) => {
  const [sort, setSort] = useState<SortState>(null);
  const [copied, setCopied] = useState(false);
  const [filter, setFilter] = useState('');

  const filteredRows = useMemo(() => {
    const needle = filter.trim().toLowerCase();
    if (!needle) return rows;
    return rows.filter((row) => row.some((value) => value.toLowerCase().includes(needle)));
  }, [rows, filter]);

  const sortedRows = useMemo(() => {
    if (!sort) return filteredRows;
    const { column, direction } = sort;
    const sign = direction === 'asc' ? 1 : -1;
    return [...rows].sort((a, b) => {
      const left = a[column] ?? '';
      const right = b[column] ?? '';
      const numeric = Number(left) - Number(right);
      const order =
        left !== '' &&
        right !== '' &&
        Number.isFinite(numeric) &&
        (left.trim() !== '' || right.trim() !== '')
          ? numeric
          : left.localeCompare(right);
      return order === 0 ? 0 : order * sign;
    });
  }, [filteredRows, sort]);

  const toggleSort = (column: number) =>
    setSort((current) =>
      current?.column === column
        ? current.direction === 'asc'
          ? { column, direction: 'desc' }
          : null
        : { column, direction: 'asc' },
    );

  return (
    <div data-component='TableOutput' className={OUTPUT_SURFACE}>
      <div className='mb-2 flex items-center justify-end gap-2'>
        <div className='relative'>
          <LuSearch
            aria-hidden='true'
            className='pointer-events-none absolute top-1/2 left-1.5 h-3 w-3 -translate-y-1/2 text-neutral-500'
          />
          <input
            type='search'
            aria-label='Filter table rows'
            placeholder='Filter…'
            value={filter}
            onChange={(event) => setFilter(event.target.value)}
            className='w-36 rounded border border-neutral-700 bg-neutral-800 py-0.5 pr-2 pl-6 text-[11px] text-neutral-200 outline-none placeholder:text-neutral-500 focus:border-neutral-500'
          />
        </div>
        <button
          type='button'
          className='flex items-center gap-1 rounded px-1.5 py-0.5 text-[11px] text-neutral-400 hover:bg-neutral-800 hover:text-neutral-200'
          onClick={() => {
            void navigator.clipboard
              .writeText(toCsv(columns, sortedRows))
              .then(() => {
                setCopied(true);
                window.setTimeout(() => setCopied(false), 1500);
              })
              .catch(() => {});
          }}>
          {copied ? 'Copied' : 'Copy CSV'}
          <LuCopy aria-hidden='true' className='h-3 w-3' />
        </button>
      </div>
      <table className='w-full border-collapse text-sm'>
        <thead>
          <tr>
            {columns.map((column, index) => (
              <th
                key={index}
                aria-sort={
                  sort?.column === index
                    ? sort.direction === 'asc'
                      ? 'ascending'
                      : 'descending'
                    : 'none'
                }
                className='cursor-pointer border border-neutral-700 bg-neutral-800 p-2 text-left text-neutral-100 select-none hover:bg-neutral-700'
                onClick={() => toggleSort(index)}>
                <span className='inline-flex items-center gap-1'>
                  {column}
                  {sort?.column === index ? (
                    sort.direction === 'asc' ? (
                      <LuArrowUp aria-hidden='true' className='h-3 w-3' />
                    ) : (
                      <LuArrowDown aria-hidden='true' className='h-3 w-3' />
                    )
                  ) : null}
                </span>
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {sortedRows.map((row, rowIndex) => (
            <tr key={rowIndex}>
              {row.map((value, columnIndex) => (
                <td
                  key={columnIndex}
                  className='border border-neutral-700 bg-neutral-900 p-2 text-left text-neutral-300'>
                  {value}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};

TableOutput.displayName = 'TableOutput';
export default TableOutput;
