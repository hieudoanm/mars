import type { FC } from 'react';
import type { LabDataset } from '../../lib/kernelService';
import { Badge } from '../shared';

type LabDatasetListProps = {
  datasets: LabDataset[];
  onSelect?: (dataset: LabDataset) => void;
};

const formatBytes = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
};

const LabDatasetList: FC<LabDatasetListProps> = ({ datasets, onSelect }) => {
  if (datasets.length === 0) {
    return (
      <p className='rounded-lg border border-dashed border-neutral-800 p-4 text-sm text-neutral-500'>
        No datasets yet. Upload a CSV to get started.
      </p>
    );
  }
  return (
    <ul className='space-y-2'>
      {datasets.map((dataset) => (
        <li key={dataset.id}>
          <button
            type='button'
            onClick={() => onSelect?.(dataset)}
            className='w-full rounded-lg border border-neutral-800 bg-neutral-900/60 px-3 py-2 text-left transition-colors hover:border-neutral-700 hover:bg-neutral-900'>
            <div className='flex items-center gap-2'>
              <span className='truncate text-sm text-neutral-200' title={dataset.originalFilename}>
                {dataset.originalFilename}
              </span>
              <Badge tone='neutral'>{dataset.columns.length} cols</Badge>
              <Badge tone='neutral'>{dataset.rowCount.toLocaleString()} rows</Badge>
              <span className='ml-auto text-xs text-neutral-500'>
                {formatBytes(dataset.byteSize)}
              </span>
            </div>
            <p className='mt-1 truncate text-xs text-neutral-500'>
              {dataset.columns.map((c) => c.name).join(', ')}
            </p>
          </button>
        </li>
      ))}
    </ul>
  );
};

LabDatasetList.displayName = 'LabDatasetList';
export default LabDatasetList;
