import { useMemo, useState, type FC } from 'react';
import type {
  LabAlgorithm,
  LabDataset,
  LabRLColumns,
  LabTrainRequest,
} from '../../lib/kernelService';
import { Button } from '../shared';
import LabAlgorithmPicker from './LabAlgorithmPicker';
import LabColumnMapping from './LabColumnMapping';

const TASK_TYPES = [
  { id: 'classification', label: 'Classification', needsLabel: true },
  { id: 'regression', label: 'Regression', needsLabel: true },
  { id: 'semi_supervised', label: 'Semi-supervised', needsLabel: true },
  { id: 'clustering', label: 'Clustering', needsLabel: false },
  { id: 'anomaly_detection', label: 'Anomaly detection', needsLabel: false },
  { id: 'reinforcement_learning', label: 'Reinforcement learning', needsLabel: false },
] as const;

type LabJobFormProps = {
  datasets: LabDataset[];
  algorithms: LabAlgorithm[];
  training: boolean;
  initialDatasetId: string | null;
  onUploadDataset: () => void;
  onTrain: (request: LabTrainRequest) => void;
};

const selectClass =
  'h-8 w-full rounded-lg border border-neutral-700 bg-neutral-900 px-2 text-sm text-neutral-200';

const LabJobForm: FC<LabJobFormProps> = ({
  datasets,
  algorithms,
  training,
  initialDatasetId,
  onUploadDataset,
  onTrain,
}) => {
  const [datasetId, setDatasetId] = useState(initialDatasetId ?? datasets[0]?.id ?? '');
  const [taskType, setTaskType] = useState<string>('classification');
  const [selectedFeatures, setSelectedFeatures] = useState<Set<string>>(new Set());
  const [labelColumn, setLabelColumn] = useState('');
  const [algorithmId, setAlgorithmId] = useState('');
  const [hyperparameters, setHyperparameters] = useState('{}');
  const [splitRatio, setSplitRatio] = useState('0.2');
  const [seed, setSeed] = useState('42');
  const [rlAction, setRlAction] = useState('');
  const [rlReward, setRlReward] = useState('');
  const [name, setName] = useState('');
  const [formError, setFormError] = useState<string | null>(null);

  const dataset = datasets.find((d) => d.id === datasetId);
  const columns = useMemo(() => dataset?.columns ?? [], [dataset]);
  const task = TASK_TYPES.find((t) => t.id === taskType) ?? TASK_TYPES[0];
  const taskAlgorithms = useMemo(
    () => algorithms.filter((a) => a.taskType === taskType),
    [algorithms, taskType],
  );
  const algorithm = taskAlgorithms.find((a) => a.id === algorithmId) ?? taskAlgorithms[0];

  const selectDataset = (id: string) => {
    setDatasetId(id);
    setSelectedFeatures(new Set());
    setLabelColumn('');
    setAlgorithmId('');
  };

  const selectTask = (id: string) => {
    setTaskType(id);
    setAlgorithmId('');
    setHyperparameters('{}');
  };

  const selectAlgorithm = (id: string) => {
    setAlgorithmId(id);
    const spec = algorithms.find((a) => a.id === id);
    if (spec) setHyperparameters(JSON.stringify(spec.defaultHyperparameters, null, 2));
  };

  const toggleFeature = (name: string) => {
    setSelectedFeatures((current) => {
      const next = new Set(current);
      if (next.has(name)) next.delete(name);
      else next.add(name);
      return next;
    });
  };

  const submit = () => {
    setFormError(null);
    if (!datasetId) return setFormError('Select or upload a dataset first.');
    if (selectedFeatures.size === 0) return setFormError('Select at least one feature column.');
    if (task.needsLabel && !labelColumn) return setFormError('Select a label column.');
    if (!algorithm) return setFormError('Select an algorithm.');
    let parsed: Record<string, unknown>;
    try {
      parsed = JSON.parse(hyperparameters || '{}') as Record<string, unknown>;
    } catch {
      return setFormError('Hyperparameters must be valid JSON.');
    }
    const ratio = Number(splitRatio);
    const seedNum = Number(seed);
    if (!Number.isFinite(ratio) || ratio <= 0 || ratio >= 1)
      return setFormError('Split ratio must be between 0 and 1.');
    if (!Number.isInteger(seedNum)) return setFormError('Seed must be an integer.');

    let rlColumns: LabRLColumns | null = null;
    if (taskType === 'reinforcement_learning') {
      if (!rlAction || !rlReward) return setFormError('Select action and reward columns.');
      rlColumns = {
        actionColumn: rlAction,
        rewardColumn: rlReward,
        nextStateColumns: [...selectedFeatures].filter((c) => c !== rlAction && c !== rlReward),
      };
    }

    onTrain({
      datasetId,
      taskType,
      algorithmId: algorithm.id,
      featureColumns: [...selectedFeatures],
      labelColumn: task.needsLabel ? labelColumn : null,
      rlColumns,
      hyperparameters: parsed,
      splitRatio: ratio,
      randomSeed: seedNum,
      name: name.trim() || null,
    });
  };

  return (
    <div className='mx-auto max-w-3xl space-y-5 p-6'>
      <section className='space-y-2'>
        <h2 className='text-sm font-medium text-neutral-300'>Dataset</h2>
        <div className='flex gap-2'>
          <select
            className={selectClass}
            value={datasetId}
            onChange={(e) => selectDataset(e.target.value)}>
            <option value=''>Select a dataset…</option>
            {datasets.map((d) => (
              <option key={d.id} value={d.id}>
                {d.originalFilename} ({d.rowCount.toLocaleString()} rows)
              </option>
            ))}
          </select>
          <Button size='sm' variant='secondary' onClick={onUploadDataset}>
            Upload CSV
          </Button>
        </div>
      </section>

      {dataset && (
        <>
          <section className='space-y-2'>
            <h2 className='text-sm font-medium text-neutral-300'>Task</h2>
            <div className='flex flex-wrap gap-2'>
              {TASK_TYPES.map((t) => (
                <Button
                  key={t.id}
                  size='sm'
                  variant={taskType === t.id ? 'primary' : 'secondary'}
                  onClick={() => selectTask(t.id)}>
                  {t.label}
                </Button>
              ))}
            </div>
          </section>

          <LabColumnMapping
            columns={columns}
            selectedFeatures={selectedFeatures}
            onToggleFeature={toggleFeature}
            needsLabel={task.needsLabel}
            labelColumn={labelColumn}
            onLabelChange={setLabelColumn}
            isReinforcement={taskType === 'reinforcement_learning'}
            rlAction={rlAction}
            rlReward={rlReward}
            onRlActionChange={setRlAction}
            onRlRewardChange={setRlReward}
          />

          <LabAlgorithmPicker
            algorithms={taskAlgorithms}
            algorithmId={algorithm?.id ?? ''}
            onSelect={selectAlgorithm}
            hyperparameters={hyperparameters}
            onHyperparametersChange={setHyperparameters}
          />

          <section className='flex flex-wrap items-end gap-3'>
            <label className='text-sm text-neutral-400'>
              Split ratio
              <input
                className={`${selectClass} mt-1 w-24`}
                value={splitRatio}
                onChange={(e) => setSplitRatio(e.target.value)}
              />
            </label>
            <label className='text-sm text-neutral-400'>
              Seed
              <input
                className={`${selectClass} mt-1 w-24`}
                value={seed}
                onChange={(e) => setSeed(e.target.value)}
              />
            </label>
            <label className='flex-1 text-sm text-neutral-400'>
              Name (optional)
              <input
                className={`${selectClass} mt-1 w-full`}
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder='My first model'
              />
            </label>
          </section>

          {formError && <p className='text-error text-sm'>{formError}</p>}
          <Button
            variant='primary'
            loading={training}
            disabled={!algorithm || taskAlgorithms.length === 0}
            onClick={submit}>
            {training ? 'Training…' : 'Train model'}
          </Button>
        </>
      )}
    </div>
  );
};

LabJobForm.displayName = 'LabJobForm';
export default LabJobForm;
