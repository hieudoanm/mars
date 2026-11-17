import { act, renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import {
  labAlgorithms,
  labDatasetInsights,
  labDatasetPreview,
  labExportArtefact,
  labGetJob,
  labListDatasets,
  labListJobsPage,
  labTrain,
  labUploadDataset,
  listenLabProgress,
  type LabAlgorithm,
  type LabDataset,
  type LabDatasetPreview,
  type LabJob,
  type LabJobDetail,
  type LabJobProgress,
} from '../../lib/kernelService';
import { useLab } from '../useLab';

vi.mock('../../lib/kernelService', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../lib/kernelService')>();
  return {
    ...actual,
    labAlgorithms: vi.fn(),
    labDatasetInsights: vi.fn(),
    labDatasetPreview: vi.fn(),
    labExportArtefact: vi.fn(),
    labGetJob: vi.fn(),
    labListDatasets: vi.fn(),
    labListJobsPage: vi.fn(),
    labTrain: vi.fn(),
    labUploadDataset: vi.fn(),
    listenLabProgress: vi.fn(),
  };
});

const dataset = (overrides: Partial<LabDataset> = {}): LabDataset => ({
  id: 'ds-1',
  originalFilename: 'data.csv',
  sha256: 'abc',
  byteSize: 100,
  storageKey: 'key',
  columns: [],
  rowCount: 10,
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

const jobDetail = (overrides: Partial<LabJobDetail> = {}): LabJobDetail => ({
  job: job(),
  metrics: null,
  artefacts: [],
  ...overrides,
});

const algorithm = (): LabAlgorithm => ({
  id: 'logistic_regression',
  name: 'Logistic Regression',
  taskType: 'classification',
  family: 'linear',
  description: 'desc',
  defaultHyperparameters: {},
  exports: ['csv'],
});

const preview = (): LabDatasetPreview => ({
  columns: ['a', 'b'],
  rows: [
    ['1', '2'],
    ['3', null],
  ],
});

const mockHappyPath = () => {
  vi.mocked(labListDatasets).mockResolvedValue([dataset()]);
  vi.mocked(labListJobsPage).mockResolvedValue({ items: [job()], nextCursor: 'cursor-1' });
  vi.mocked(labAlgorithms).mockResolvedValue([algorithm()]);
};

describe('useLab', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockHappyPath();
  });

  it('loads datasets, jobs, and algorithms on mount', async () => {
    const { result } = renderHook(() => useLab());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.datasets).toHaveLength(1);
    expect(result.current.jobs).toHaveLength(1);
    expect(result.current.algorithms).toHaveLength(1);
    expect(result.current.nextCursor).toBe('cursor-1');
    expect(result.current.error).toBeNull();
  });

  it('surfaces a load error', async () => {
    vi.mocked(labListDatasets).mockRejectedValue(new Error('disk full'));
    const { result } = renderHook(() => useLab());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.error).toBe('disk full');
  });

  it('surfaces a non-Error load failure', async () => {
    vi.mocked(labListDatasets).mockRejectedValue('nope');
    const { result } = renderHook(() => useLab());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.error).toBe('Failed to load Mars Lab data');
  });

  it('recovers when algorithms fail to load', async () => {
    vi.mocked(labAlgorithms).mockRejectedValue(new Error('algorithms down'));
    const { result } = renderHook(() => useLab());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.algorithms).toEqual([]);
    expect(result.current.datasets).toHaveLength(1);
  });

  it('applies job filters and reloads the first page', async () => {
    const { result } = renderHook(() => useLab());
    await waitFor(() => expect(result.current.loading).toBe(false));

    vi.mocked(labListJobsPage).mockResolvedValue({
      items: [job({ id: 'job-2' })],
      nextCursor: null,
    });
    await act(async () => {
      result.current.applyJobFilters({ status: 'failed', taskType: 'regression', q: '  rf  ' });
    });

    expect(labListJobsPage).toHaveBeenLastCalledWith(
      { statuses: ['failed'], taskTypes: ['regression'], q: 'rf' },
      null,
      25,
    );
    await waitFor(() => expect(result.current.jobs[0].id).toBe('job-2'));
    expect(result.current.nextCursor).toBeNull();
  });

  it('maps empty filters to empty lists and null query', async () => {
    const { result } = renderHook(() => useLab());
    await waitFor(() => expect(result.current.loading).toBe(false));
    await act(async () => {
      result.current.applyJobFilters({ status: '', taskType: '', q: '   ' });
    });
    expect(labListJobsPage).toHaveBeenLastCalledWith(
      { statuses: [], taskTypes: [], q: null },
      null,
      25,
    );
  });

  it('sets an error when applying filters fails', async () => {
    const { result } = renderHook(() => useLab());
    await waitFor(() => expect(result.current.loading).toBe(false));
    vi.mocked(labListJobsPage).mockRejectedValue(new Error('filter boom'));
    await act(async () => {
      result.current.applyJobFilters({ status: 'x', taskType: '', q: '' });
    });
    await waitFor(() => expect(result.current.error).toBe('filter boom'));
    expect(result.current.loading).toBe(false);
  });

  it('loads more jobs and appends them', async () => {
    const { result } = renderHook(() => useLab());
    await waitFor(() => expect(result.current.loading).toBe(false));

    vi.mocked(labListJobsPage).mockResolvedValue({
      items: [job({ id: 'job-2' })],
      nextCursor: null,
    });
    await act(async () => {
      await result.current.loadMoreJobs();
    });
    expect(result.current.jobs.map((j) => j.id)).toEqual(['job-1', 'job-2']);
    expect(result.current.nextCursor).toBeNull();
    expect(result.current.loadingMore).toBe(false);
  });

  it('does nothing when there is no cursor', async () => {
    vi.mocked(labListJobsPage).mockResolvedValue({ items: [job()], nextCursor: null });
    const { result } = renderHook(() => useLab());
    await waitFor(() => expect(result.current.loading).toBe(false));
    const calls = vi.mocked(labListJobsPage).mock.calls.length;
    await act(async () => {
      await result.current.loadMoreJobs();
    });
    expect(vi.mocked(labListJobsPage).mock.calls.length).toBe(calls);
  });

  it('sets an error when loading more fails', async () => {
    const { result } = renderHook(() => useLab());
    await waitFor(() => expect(result.current.loading).toBe(false));
    vi.mocked(labListJobsPage).mockRejectedValue(new Error('page boom'));
    await act(async () => {
      await result.current.loadMoreJobs();
    });
    expect(result.current.error).toBe('page boom');
    expect(result.current.loadingMore).toBe(false);
  });

  it('uploads a dataset and prepends it', async () => {
    const { result } = renderHook(() => useLab());
    await waitFor(() => expect(result.current.loading).toBe(false));
    vi.mocked(labUploadDataset).mockResolvedValue({
      dataset: dataset({ id: 'ds-2', originalFilename: 'new.csv' }),
      profile: { rowCount: 1, columnCount: 1, totalMissing: 0, columns: [], warnings: [] },
    });
    let uploaded: LabDataset | null | undefined;
    await act(async () => {
      uploaded = await result.current.uploadDataset('/tmp/new.csv');
    });
    expect(uploaded?.id).toBe('ds-2');
    expect(result.current.datasets[0].id).toBe('ds-2');
  });

  it('returns null and sets an error when upload fails', async () => {
    const { result } = renderHook(() => useLab());
    await waitFor(() => expect(result.current.loading).toBe(false));
    vi.mocked(labUploadDataset).mockRejectedValue(new Error('upload boom'));
    let uploaded: LabDataset | null | undefined;
    await act(async () => {
      uploaded = await result.current.uploadDataset('/tmp/x.csv');
    });
    expect(uploaded).toBeNull();
    expect(result.current.error).toBe('upload boom');
  });

  it('shows a dataset preview with extracted warnings', async () => {
    const { result } = renderHook(() => useLab());
    await waitFor(() => expect(result.current.loading).toBe(false));
    vi.mocked(labDatasetPreview).mockResolvedValue(preview());
    vi.mocked(labDatasetInsights).mockResolvedValue({
      warnings: [{ code: 'W1', column: 'a', message: 'sparse column' }],
    });

    await act(async () => {
      await result.current.showDatasetPreview(dataset());
    });
    expect(result.current.datasetPreview?.dataset.id).toBe('ds-1');
    expect(result.current.datasetPreview?.preview.columns).toEqual(['a', 'b']);
    expect(result.current.datasetPreview?.warnings).toHaveLength(1);
    expect(result.current.datasetPreview?.warnings[0].message).toBe('sparse column');

    act(() => result.current.closeDatasetPreview());
    expect(result.current.datasetPreview).toBeNull();
  });

  it('treats non-object insights as no warnings', async () => {
    const { result } = renderHook(() => useLab());
    await waitFor(() => expect(result.current.loading).toBe(false));
    vi.mocked(labDatasetPreview).mockResolvedValue(preview());
    vi.mocked(labDatasetInsights).mockResolvedValue(null);
    await act(async () => {
      await result.current.showDatasetPreview(dataset());
    });
    expect(result.current.datasetPreview?.warnings).toEqual([]);
  });

  it('sets an error when the preview fails', async () => {
    const { result } = renderHook(() => useLab());
    await waitFor(() => expect(result.current.loading).toBe(false));
    vi.mocked(labDatasetPreview).mockRejectedValue(new Error('preview boom'));
    await act(async () => {
      await result.current.showDatasetPreview(dataset());
    });
    expect(result.current.error).toBe('preview boom');
    expect(result.current.datasetPreview).toBeNull();
  });

  it('selects a job by id', async () => {
    const { result } = renderHook(() => useLab());
    await waitFor(() => expect(result.current.loading).toBe(false));
    vi.mocked(labGetJob).mockResolvedValue(jobDetail({ job: job({ id: 'job-9' }) }));
    await act(async () => {
      await result.current.selectJob('job-9');
    });
    expect(result.current.selectedJob?.job.id).toBe('job-9');
  });

  it('clears selection and sets an error when selectJob fails', async () => {
    const { result } = renderHook(() => useLab());
    await waitFor(() => expect(result.current.loading).toBe(false));
    vi.mocked(labGetJob).mockRejectedValue(new Error('job boom'));
    await act(async () => {
      await result.current.selectJob('missing');
    });
    expect(result.current.selectedJob).toBeNull();
    expect(result.current.error).toBe('job boom');
  });

  it('loads comparable job details and drops nulls', async () => {
    const { result } = renderHook(() => useLab());
    await waitFor(() => expect(result.current.loading).toBe(false));
    vi.mocked(labGetJob)
      .mockResolvedValueOnce(jobDetail({ job: job({ id: 'a' }) }))
      .mockResolvedValueOnce(null)
      .mockResolvedValueOnce(jobDetail({ job: job({ id: 'c' }) }));
    await act(async () => {
      await result.current.loadCompare(['a', 'b', 'c']);
    });
    expect(result.current.compareDetails.map((d) => d.job.id)).toEqual(['a', 'c']);
  });

  it('sets an error when compare fails', async () => {
    const { result } = renderHook(() => useLab());
    await waitFor(() => expect(result.current.loading).toBe(false));
    vi.mocked(labGetJob).mockRejectedValue(new Error('compare boom'));
    await act(async () => {
      await result.current.loadCompare(['a']);
    });
    expect(result.current.error).toBe('compare boom');
    expect(result.current.compareDetails).toEqual([]);
  });

  it('trains a model, tracks progress, and selects the result', async () => {
    const { result } = renderHook(() => useLab());
    await waitFor(() => expect(result.current.loading).toBe(false));

    let progressHandler: ((event: LabJobProgress) => void) | undefined;
    const unlisten = vi.fn();
    vi.mocked(listenLabProgress).mockImplementation(async (handler) => {
      progressHandler = handler;
      return unlisten;
    });
    vi.mocked(labTrain).mockImplementation(async () => {
      progressHandler?.({ jobId: 'job-1', stage: 'training' });
      progressHandler?.({ jobId: 'job-1', stage: 'completed' });
      return jobDetail({ job: job({ id: 'job-new' }) });
    });

    const request = {
      datasetId: 'ds-1',
      taskType: 'classification',
      algorithmId: 'logistic_regression',
      featureColumns: ['a'],
      labelColumn: 'target',
      rlColumns: null,
      hyperparameters: {},
      splitRatio: 0.2,
      randomSeed: 42,
      name: null,
    };

    let detail: LabJobDetail | null | undefined;
    await act(async () => {
      detail = await result.current.train(request);
    });

    expect(detail?.job.id).toBe('job-new');
    expect(result.current.selectedJob?.job.id).toBe('job-new');
    expect(result.current.jobs[0].id).toBe('job-new');
    expect(result.current.training).toBe(false);
    expect(result.current.trainingStage).toBeNull();
    expect(unlisten).toHaveBeenCalled();
  });

  it('refreshes and reports an error when training fails', async () => {
    const { result } = renderHook(() => useLab());
    await waitFor(() => expect(result.current.loading).toBe(false));
    vi.mocked(listenLabProgress).mockResolvedValue(() => {});
    vi.mocked(labTrain).mockRejectedValue(new Error('train boom'));

    let detail: LabJobDetail | null | undefined;
    await act(async () => {
      detail = await result.current.train({
        datasetId: 'ds-1',
        taskType: 'classification',
        algorithmId: 'a',
        featureColumns: [],
        labelColumn: null,
        rlColumns: null,
        hyperparameters: {},
        splitRatio: 0.2,
        randomSeed: 1,
        name: null,
      });
    });
    expect(detail).toBeNull();
    expect(result.current.error).toBe('train boom');
    expect(result.current.training).toBe(false);
    // refresh() re-fetched datasets/jobs after the failure
    expect(labListDatasets).toHaveBeenCalledTimes(2);
  });

  it('exports an artefact successfully', async () => {
    const { result } = renderHook(() => useLab());
    await waitFor(() => expect(result.current.loading).toBe(false));
    vi.mocked(labExportArtefact).mockResolvedValue(undefined);
    let ok: boolean | undefined;
    await act(async () => {
      ok = await result.current.exportJob('job-1', 'csv', '/tmp/out.csv');
    });
    expect(ok).toBe(true);
    expect(labExportArtefact).toHaveBeenCalledWith('job-1', 'csv', '/tmp/out.csv');
  });

  it('returns false and sets an error when export fails', async () => {
    const { result } = renderHook(() => useLab());
    await waitFor(() => expect(result.current.loading).toBe(false));
    vi.mocked(labExportArtefact).mockRejectedValue(new Error('export boom'));
    let ok: boolean | undefined;
    await act(async () => {
      ok = await result.current.exportJob('job-1', 'csv', '/tmp/out.csv');
    });
    expect(ok).toBe(false);
    expect(result.current.error).toBe('export boom');
  });
});
