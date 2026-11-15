import type { FC } from 'react';
import type { LabAlgorithm } from '../../lib/kernelService';

const selectClass =
  'h-8 w-full rounded-lg border border-neutral-700 bg-neutral-900 px-2 text-sm text-neutral-200';

type LabAlgorithmPickerProps = {
  algorithms: LabAlgorithm[];
  algorithmId: string;
  onSelect: (id: string) => void;
  hyperparameters: string;
  onHyperparametersChange: (value: string) => void;
};

const LabAlgorithmPicker: FC<LabAlgorithmPickerProps> = ({
  algorithms,
  algorithmId,
  onSelect,
  hyperparameters,
  onHyperparametersChange,
}) => {
  const algorithm = algorithms.find((a) => a.id === algorithmId) ?? algorithms[0];
  return (
    <section className='space-y-2'>
      <h2 className='text-sm font-medium text-neutral-300'>Algorithm</h2>
      <select
        className={selectClass}
        value={algorithm?.id ?? ''}
        onChange={(e) => onSelect(e.target.value)}>
        {algorithms.map((a) => (
          <option key={a.id} value={a.id}>
            {a.name} — {a.description}
          </option>
        ))}
      </select>
      {algorithm && (
        <details className='rounded-lg border border-neutral-800 p-3'>
          <summary className='cursor-pointer text-xs text-neutral-400'>
            Hyperparameters (JSON)
          </summary>
          <textarea
            className='mt-2 h-28 w-full rounded-md border border-neutral-700 bg-neutral-950 p-2 font-mono text-xs text-neutral-200'
            value={hyperparameters}
            onChange={(e) => onHyperparametersChange(e.target.value)}
            spellCheck={false}
          />
        </details>
      )}
    </section>
  );
};

LabAlgorithmPicker.displayName = 'LabAlgorithmPicker';
export default LabAlgorithmPicker;
