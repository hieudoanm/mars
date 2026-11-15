import { useCallback, useEffect, useRef, useState } from 'react';
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
  type LabJobFilters,
  type LabTrainRequest,
} from '../lib/kernelService';

export type LabRouteView =
  | { kind: 'home' }
  | { kind: 'new-job' }
  | { kind: 'job'; jobId: string }
  | { kind: 'compare'; jobIds: string[] };

export type LabJobFilterState = {
  status: string;
  taskType: string;
  q: string;
};

const PAGE_SIZE = 25;

const toFilters = (state: LabJobFilterState): LabJobFilters => ({
  statuses: state.status ? [state.status] : [],
  taskTypes: state.taskType ? [state.taskType] : [],
  q: state.q.trim() || null,
});

export const useLab = () => {
  const [datasets, setDatasets] = useState<LabDataset[]>([]);
  const [jobs, setJobs] = useState<LabJob[]>([]);
  const [nextCursor, setNextCursor] = useState<string | null>(null);
  const [loadingMore, setLoadingMore] = useState(false);
  const [algorithms, setAlgorithms] = useState<LabAlgorithm[]>([]);
  const [selectedJob, setSelectedJob] = useState<LabJobDetail | null>(null);
  const [compareDetails, setCompareDetails] = useState<LabJobDetail[]>([]);
  const [datasetPreview, setDatasetPreview] = useState<{
    dataset: LabDataset;
    preview: LabDatasetPreview;
    warnings: { code?: string; column?: string | null; message: string }[];
  } | null>(null);
  const [jobFilters, setJobFilters] = useState<LabJobFilterState>({
    status: '',
    taskType: '',
    q: '',
  });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [training, setTraining] = useState(false);
  const [trainingStage, setTrainingStage] = useState<string | null>(null);
  const filtersRef = useRef(jobFilters);
  filtersRef.current = jobFilters;

  const refresh = useCallback(async () => {
    try {
      const [datasetList, page] = await Promise.all([
        labListDatasets(),
        labListJobsPage(toFilters(filtersRef.current), null, PAGE_SIZE),
      ]);
      setDatasets(datasetList);
      setJobs(page.items);
      setNextCursor(page.nextCursor);
      setError(null);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Failed to load Mars Lab data');
    } finally {
      setLoading(false);
    }
  }, []);

  const applyJobFilters = useCallback(
    (next: LabJobFilterState) => {
      setJobFilters(next);
      setLoading(true);
      void labListJobsPage(toFilters(next), null, PAGE_SIZE)
        .then((page) => {
          setJobs(page.items);
          setNextCursor(page.nextCursor);
          setError(null);
        })
        .catch((cause) =>
          setError(cause instanceof Error ? cause.message : 'Failed to load jobs'),
        )
        .finally(() => setLoading(false));
    },
    [],
  );

  const loadMoreJobs = useCallback(async () => {
    if (!nextCursor || loadingMore) return;
    setLoadingMore(true);
    try {
      const page = await labListJobsPage(toFilters(filtersRef.current), nextCursor, PAGE_SIZE);
      setJobs((current) => [...current, ...page.items]);
      setNextCursor(page.nextCursor);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Failed to load more jobs');
    } finally {
      setLoadingMore(false);
    }
  }, [nextCursor, loadingMore]);

  useEffect(() => {
    void refresh();
    void labAlgorithms()
      .then(setAlgorithms)
      .catch(() => setAlgorithms([]));
  }, [refresh]);

  const uploadDataset = useCallback(async (path: string) => {
    setError(null);
    try {
      const detail = await labUploadDataset(path);
      setDatasets((current) => [detail.dataset, ...current]);
      return detail.dataset;
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Dataset upload failed');
      return null;
    }
  }, []);

  const showDatasetPreview = useCallback(async (dataset: LabDataset) => {
    setError(null);
    try {
      const [preview, insight] = await Promise.all([
        labDatasetPreview(dataset.id),
        labDatasetInsights(dataset.id),
      ]);
      const warnings =
        insight !== null &&
        typeof insight === 'object' &&
        Array.isArray((insight as { warnings?: unknown }).warnings)
          ? ((insight as { warnings: { code?: string; column?: string | null; message: string }[] })
              .warnings)
          : [];
      setDatasetPreview({ dataset, preview, warnings });
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Failed to load dataset preview');
    }
  }, []);

  const closeDatasetPreview = useCallback(() => setDatasetPreview(null), []);

  const selectJob = useCallback(async (jobId: string) => {
    setSelectedJob(null);
    try {
      setSelectedJob(await labGetJob(jobId));
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Failed to load job');
    }
  }, []);

  const loadCompare = useCallback(async (jobIds: string[]) => {
    setCompareDetails([]);
    try {
      const details = await Promise.all(jobIds.map((id) => labGetJob(id)));
      setCompareDetails(details.filter((d): d is LabJobDetail => d !== null));
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Failed to load jobs for comparison');
    }
  }, []);

  const train = useCallback(
    async (request: LabTrainRequest) => {
      setTraining(true);
      setTrainingStage('queued');
      setError(null);
      let unlisten: (() => void) | undefined;
      try {
        unlisten = await listenLabProgress((event) => {
          if (event.stage !== 'completed' && event.stage !== 'failed') {
            setTrainingStage(event.stage);
          }
        });
        const detail = await labTrain(request);
        setJobs((current) => [detail.job, ...current.filter((j) => j.id !== detail.job.id)]);
        setSelectedJob(detail);
        return detail;
      } catch (cause) {
        setError(cause instanceof Error ? cause.message : 'Training failed');
        await refresh();
        return null;
      } finally {
        unlisten?.();
        setTraining(false);
        setTrainingStage(null);
      }
    },
    [refresh],
  );

  const exportJob = useCallback(async (jobId: string, format: string, outputPath: string) => {
    try {
      await labExportArtefact(jobId, format, outputPath);
      return true;
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Export failed');
      return false;
    }
  }, []);

  return {
    datasets,
    jobs,
    nextCursor,
    loadingMore,
    algorithms,
    selectedJob,
    compareDetails,
    datasetPreview,
    jobFilters,
    loading,
    error,
    training,
    trainingStage,
    refresh,
    uploadDataset,
    showDatasetPreview,
    closeDatasetPreview,
    selectJob,
    loadCompare,
    applyJobFilters,
    loadMoreJobs,
    train,
    exportJob,
  };
};
