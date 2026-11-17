import type { FC } from 'react';
import type { LabDataset, LabDatasetPreview } from '../../lib/kernelService';
import { IconButton } from '../shared';

type LabDatasetPreviewDialogProps = {
  dataset: LabDataset;
  preview: LabDatasetPreview | null;
  warnings?: { code?: string; column?: string | null; message: string }[];
  onClose: () => void;
};

const LabDatasetPreviewDialog: FC<LabDatasetPreviewDialogProps> = ({
  dataset,
  preview,
  warnings = [],
  onClose,
}) => {
  return (
    <div
      data-component='LabDatasetPreviewDialog'
      className='fixed inset-0 z-50 grid place-items-center bg-black/65'
      role='presentation'
      onClick={onClose}>
      <section
        className='w-[42rem] max-w-[90vw] rounded-xl border border-neutral-800 bg-neutral-900 p-5 shadow-2xl'
        role='dialog'
        aria-modal='true'
        aria-labelledby='lab-dataset-preview-title'
        onClick={(e) => e.stopPropagation()}>
        <div className='flex items-start justify-between gap-4'>
          <div>
            <h2 id='lab-dataset-preview-title' className='text-sm font-semibold text-neutral-200'>
              {dataset.originalFilename}
            </h2>
            <p className='mt-1 text-xs text-neutral-500'>
              {dataset.rowCount.toLocaleString()} rows · {dataset.columns.length} columns
            </p>
          </div>
          <IconButton label='Close preview' icon='✕' onClick={onClose} />
        </div>
        {warnings.length > 0 && (
          <ul className='mt-3 space-y-1'>
            {warnings.map((warning, i) => (
              <li
                key={i}
                className='border-warning/30 bg-warning/10 text-warning rounded-md border px-3 py-1.5 text-xs'>
                {warning.column ? `${warning.column}: ` : ''}
                {warning.message}
              </li>
            ))}
          </ul>
        )}
        {preview ? (
          <>
            <div className='mt-3 max-h-72 overflow-auto rounded-lg border border-neutral-800'>
              <table className='w-full text-xs'>
                <thead className='sticky top-0 bg-neutral-900'>
                  <tr>
                    {preview.columns.map((column) => (
                      <th
                        key={column}
                        className='px-2 py-1.5 text-left font-medium text-neutral-400'>
                        {column}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {preview.rows.map((row, i) => (
                    <tr key={i} className='border-t border-neutral-800/60'>
                      {row.map((cell, j) => (
                        <td key={j} className='max-w-40 truncate px-2 py-1 text-neutral-300'>
                          {cell ?? <span className='text-neutral-600'>null</span>}
                        </td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
            <p className='mt-2 text-xs text-neutral-500'>
              Showing first {preview.rows.length} rows
            </p>
          </>
        ) : (
          <p className='mt-3 text-sm text-neutral-500'>Loading preview…</p>
        )}
      </section>
    </div>
  );
};

LabDatasetPreviewDialog.displayName = 'LabDatasetPreviewDialog';
export default LabDatasetPreviewDialog;
