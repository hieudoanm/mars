import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { open as openFileDialog, save as saveFileDialog } from '@tauri-apps/plugin-dialog';
import LabView from '../LabView';
import type { LabRouteView } from '../../../hooks/useLab';
import type { LabAlgorithm, LabDataset, LabJob, LabJobDetail } from '../../../lib/kernelService';

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
  save: vi.fn(),
}));

const mocks = vi.hoisted(() => ({
  lab: null as Record<string, unknown> | null,
}));

vi.mock('../../../hooks/useLab', () => ({
  useLab: () => mocks.lab,
}));

const dataset = (overrides: Partial<LabDataset> = {}): LabDataset => ({
  id: 'ds-1',
  originalFilename: 'data.csv',
  sha256: 'abc',
  byteSize: 2048,
  storageKey: 'key',
  columns: [
    { name: 'a', inferredDtype: 'f64', nullCount: 0, sampleValues: [] },
    { name: 'b', inferredDtype: 'i64', nullCount: 0, sampleValues: [] },
  ],
  rowCount: 1500,
  createdAt: '2026-01-01T00:00:00Z',
  ...overrides,
});

const job = (overrides: Partial<LabJob> = {}): LabJob => ({
  id: 'job-1',
  datasetId: 'ds-1',
  taskType: 'classification',
  algorithmId: 'logistic_regression',
  featureColumns: ['a'],
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

const algorithm = (): LabAlgorithm => ({
  id: 'logistic_regression',
  name: 'Logistic Regression',
  taskType: 'classification',
  family: 'linear',
  description: 'desc',
  defaultHyperparameters: {},
  exports: ['onnx'],
});

const labStub = (overrides: Record<string, unknown> = {}) => ({
  datasets: [dataset()],
  jobs: [job()],
  nextCursor: null,
  loadingMore: false,
  algorithms: [algorithm()],
  selectedJob: null,
  compareDetails: [],
  datasetPreview: null,
  jobFilters: { status: '', taskType: '', q: '' },
  loading: false,
  error: null,
  training: false,
  trainingStage: null,
  refresh: vi.fn(),
  uploadDataset: vi.fn().mockResolvedValue(dataset()),
  showDatasetPreview: vi.fn(),
  closeDatasetPreview: vi.fn(),
  selectJob: vi.fn(),
  loadCompare: vi.fn(),
  applyJobFilters: vi.fn(),
  loadMoreJobs: vi.fn(),
  train: vi.fn(),
  exportJob: vi.fn().mockResolvedValue(true),
  ...overrides,
});

const renderView = (view: LabRouteView = { kind: 'home' }, onNavigate = vi.fn()) => {
  const utils = render(<LabView view={view} onNavigate={onNavigate} onHome={vi.fn()} />);
  return { ...utils, onNavigate };
};

describe('LabView', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.lab = labStub();
  });

  it('shows a loading state', () => {
    mocks.lab = labStub({ loading: true });
    renderView();
    expect(screen.getByText('Loading Mars Lab…')).toBeInTheDocument();
  });

  it('shows an error banner', () => {
    mocks.lab = labStub({ error: 'kernel exploded' });
    renderView();
    expect(screen.getByText('kernel exploded')).toBeInTheDocument();
  });

  it('renders datasets and jobs on the home view', () => {
    renderView();
    expect(screen.getByText('Datasets')).toBeInTheDocument();
    expect(screen.getByText('Training jobs')).toBeInTheDocument();
    expect(screen.getByText('data.csv')).toBeInTheDocument();
    expect(screen.getByText('logistic_regression')).toBeInTheDocument();
  });

  it('shows the empty dataset state', () => {
    mocks.lab = labStub({ datasets: [] });
    renderView();
    expect(screen.getByText(/No datasets yet/)).toBeInTheDocument();
  });

  it('navigates to the new-job view and back home', () => {
    const { onNavigate } = renderView();
    fireEvent.click(screen.getByRole('button', { name: 'New training job' }));
    expect(onNavigate).toHaveBeenCalledWith({ kind: 'new-job' });
  });

  it('shows a back button on non-home views', () => {
    const { onNavigate } = renderView({ kind: 'new-job' });
    fireEvent.click(screen.getByRole('button', { name: '← Back' }));
    expect(onNavigate).toHaveBeenCalledWith({ kind: 'home' });
  });

  it('renders the training stage on the primary button', () => {
    mocks.lab = labStub({ training: true, trainingStage: 'training' });
    renderView();
    expect(screen.getByRole('button', { name: /Training: training/ })).toBeInTheDocument();
  });

  it('previews a dataset when its row is clicked', () => {
    renderView();
    fireEvent.click(screen.getByText('data.csv'));
    expect(mocks.lab!.showDatasetPreview).toHaveBeenCalledWith(
      expect.objectContaining({ id: 'ds-1' }),
    );
  });

  it('uploads a CSV picked through the file dialog', async () => {
    vi.mocked(openFileDialog).mockResolvedValue('/tmp/upload.csv');
    renderView();
    fireEvent.click(screen.getByRole('button', { name: 'Upload CSV' }));
    await waitFor(() => expect(mocks.lab!.uploadDataset).toHaveBeenCalledWith('/tmp/upload.csv'));
  });

  it('does nothing when the file dialog is cancelled', async () => {
    vi.mocked(openFileDialog).mockResolvedValue(null);
    renderView();
    fireEvent.click(screen.getByRole('button', { name: 'Upload CSV' }));
    await waitFor(() => expect(openFileDialog).toHaveBeenCalled());
    expect(mocks.lab!.uploadDataset).not.toHaveBeenCalled();
  });

  it('applies status, task, and search filters', () => {
    renderView();
    fireEvent.change(screen.getByLabelText('Filter by status'), {
      target: { value: 'failed' },
    });
    expect(mocks.lab!.applyJobFilters).toHaveBeenCalledWith(
      expect.objectContaining({ status: 'failed' }),
    );
    fireEvent.change(screen.getByLabelText('Filter by task'), {
      target: { value: 'regression' },
    });
    expect(mocks.lab!.applyJobFilters).toHaveBeenLastCalledWith(
      expect.objectContaining({ taskType: 'regression' }),
    );
    fireEvent.change(screen.getByLabelText('Search jobs'), { target: { value: 'rf' } });
    expect(mocks.lab!.applyJobFilters).toHaveBeenLastCalledWith(
      expect.objectContaining({ q: 'rf' }),
    );
  });

  it('loads more jobs when a cursor exists', () => {
    mocks.lab = labStub({ nextCursor: 'cursor-1' });
    renderView();
    fireEvent.click(screen.getByRole('button', { name: 'Load more' }));
    expect(mocks.lab!.loadMoreJobs).toHaveBeenCalled();
  });

  it('navigates to a job detail when a job row is selected', () => {
    const { onNavigate } = renderView();
    fireEvent.click(screen.getByRole('button', { name: /logistic_regression/ }));
    expect(onNavigate).toHaveBeenCalledWith({ kind: 'job', jobId: 'job-1' });
  });

  it('compares two selected jobs', () => {
    mocks.lab = labStub({
      jobs: [job(), job({ id: 'job-2', algorithmId: 'random_forest' })],
    });
    const { onNavigate } = renderView();
    fireEvent.click(screen.getByRole('button', { name: 'Compare' }));
    fireEvent.click(screen.getByRole('checkbox', { name: 'Compare logistic_regression' }));
    fireEvent.click(screen.getByRole('checkbox', { name: 'Compare random_forest' }));
    fireEvent.click(screen.getByRole('button', { name: /Compare \(2\)/ }));
    expect(onNavigate).toHaveBeenCalledWith({
      kind: 'compare',
      jobIds: expect.arrayContaining(['job-1', 'job-2']),
    });
  });

  it('renders the dataset preview dialog', () => {
    mocks.lab = labStub({
      datasetPreview: {
        dataset: dataset(),
        preview: { columns: ['a'], rows: [['1']] },
        warnings: [{ message: 'sparse' }],
      },
    });
    renderView();
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    fireEvent.click(screen.getByLabelText('Close preview'));
    expect(mocks.lab!.closeDatasetPreview).toHaveBeenCalled();
  });

  it('renders job detail with artefact download', async () => {
    const detail: LabJobDetail = {
      job: job({ name: 'My model' }),
      metrics: {
        jobId: 'job-1',
        metrics: { accuracy: 0.9 },
        charts: null,
        createdAt: '2026-01-01T00:00:00Z',
      },
      artefacts: [
        {
          id: 'art-1',
          jobId: 'job-1',
          format: 'onnx',
          filename: 'model.onnx',
          byteSize: 2048,
          storageKey: 'k',
          createdAt: '2026-01-01T00:00:00Z',
        },
      ],
    };
    mocks.lab = labStub({ selectedJob: detail });
    vi.mocked(saveFileDialog).mockResolvedValue('/tmp/model.onnx');
    renderView({ kind: 'job', jobId: 'job-1' });
    expect(screen.getByText('My model')).toBeInTheDocument();
    expect(screen.getByText('model.onnx')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Download ONNX' }));
    await waitFor(() =>
      expect(mocks.lab!.exportJob).toHaveBeenCalledWith('job-1', 'onnx', '/tmp/model.onnx'),
    );
  });

  it('renders the compare view', () => {
    mocks.lab = labStub({
      compareDetails: [
        { job: job(), metrics: null, artefacts: [] },
        {
          job: job({ id: 'job-2', algorithmId: 'random_forest' }),
          metrics: null,
          artefacts: [],
        },
      ],
    });
    renderView({ kind: 'compare', jobIds: ['job-1', 'job-2'] });
    expect(screen.getByText('logistic_regression')).toBeInTheDocument();
    expect(screen.getByText('random_forest')).toBeInTheDocument();
  });
});
