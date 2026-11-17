import type { FC } from 'react';
import type { LabColumnPreview } from '../../lib/kernelService';
import { Checkbox } from '../shared';

const selectClass =
  'h-8 w-full rounded-lg border border-neutral-700 bg-neutral-900 px-2 text-sm text-neutral-200';

type LabColumnMappingProps = {
  columns: LabColumnPreview[];
  selectedFeatures: Set<string>;
  onToggleFeature: (name: string) => void;
  needsLabel: boolean;
  labelColumn: string;
  onLabelChange: (name: string) => void;
  isReinforcement: boolean;
  rlAction: string;
  rlReward: string;
  onRlActionChange: (name: string) => void;
  onRlRewardChange: (name: string) => void;
};

const LabColumnMapping: FC<LabColumnMappingProps> = ({
  columns,
  selectedFeatures,
  onToggleFeature,
  needsLabel,
  labelColumn,
  onLabelChange,
  isReinforcement,
  rlAction,
  rlReward,
  onRlActionChange,
  onRlRewardChange,
}) => {
  const columnNames = columns.map((c) => c.name);
  const labelOptions = columnNames.filter((n) => !selectedFeatures.has(n) || n === labelColumn);
  const rlSelect = (value: string, onChange: (name: string) => void, label: string) => (
    <label className='flex items-center gap-2 text-sm text-neutral-400'>
      {label}
      <select
        className={`${selectClass} max-w-56`}
        value={value}
        onChange={(e) => onChange(e.target.value)}>
        <option value=''>Select…</option>
        {columnNames.map((n) => (
          <option key={n} value={n}>
            {n}
          </option>
        ))}
      </select>
    </label>
  );

  return (
    <section className='space-y-2'>
      <h2 className='text-sm font-medium text-neutral-300'>Columns</h2>
      <div className='grid grid-cols-2 gap-1 rounded-lg border border-neutral-800 p-3 sm:grid-cols-3'>
        {columns.map((column) => (
          <Checkbox
            key={column.name}
            label={`${column.name} (${column.inferredDtype})`}
            checked={selectedFeatures.has(column.name)}
            onChange={() => onToggleFeature(column.name)}
          />
        ))}
      </div>
      {needsLabel && (
        <label className='flex items-center gap-2 text-sm text-neutral-400'>
          Label column
          <select
            className={`${selectClass} max-w-56`}
            value={labelColumn}
            onChange={(e) => onLabelChange(e.target.value)}>
            <option value=''>Select…</option>
            {labelOptions.map((n) => (
              <option key={n} value={n}>
                {n}
              </option>
            ))}
          </select>
        </label>
      )}
      {isReinforcement && (
        <div className='flex flex-wrap gap-3'>
          {rlSelect(rlAction, onRlActionChange, 'Action')}
          {rlSelect(rlReward, onRlRewardChange, 'Reward')}
        </div>
      )}
    </section>
  );
};

LabColumnMapping.displayName = 'LabColumnMapping';
export default LabColumnMapping;
