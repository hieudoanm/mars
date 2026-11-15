import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import LabJobList from '../LabJobList';
import type { LabJob } from '../../../lib/kernelService';

const job = (overrides: Partial<LabJob> = {}): LabJob => ({
  id: 'job-1',
  datasetId: 'ds-1',
  taskType: 'classification',
  algorithmId: 'logistic_regression',
  featureColumns: ['a', 'b'],
  labelColumn: 'target',
  rlColumns: null,
  hyperparameters: {},
  splitRatio: 0.2,
  randomSeed: 42,
  status: 'completed',
  errorCode: null,
  errorMessage: null,
  name: null,
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: '2026-01-01T00:00:00Z',
  ...overrides,
});

describe('LabJobList', () => {
  it('renders the empty state', () => {
    render(<LabJobList jobs={[]} onSelect={() => {}} />);
    expect(screen.getByText('No training jobs yet.')).toBeInTheDocument();
  });

  it('renders job rows with status badges', () => {
    render(
      <LabJobList
        jobs={[
          job(),
          job({ id: 'job-2', status: 'failed', algorithmId: 'random_forest', errorMessage: 'boom' }),
        ]}
        onSelect={() => {}}
      />,
    );
    expect(screen.getByText('logistic_regression')).toBeInTheDocument();
    expect(screen.getByText('completed')).toBeInTheDocument();
    expect(screen.getByText('failed')).toBeInTheDocument();
    expect(screen.getByText('boom')).toBeInTheDocument();
  });

  it('calls onSelect with the job id', () => {
    const onSelect = vi.fn();
    render(<LabJobList jobs={[job()]} onSelect={onSelect} />);
    fireEvent.click(screen.getByRole('button', { name: /logistic_regression/ }));
    expect(onSelect).toHaveBeenCalledWith('job-1');
  });
});
