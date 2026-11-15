import { useCallback, useState, type FC } from 'react';
import { open as openFileDialog, save as saveFileDialog } from '@tauri-apps/plugin-dialog';
import { Button } from '../shared';
import NotebookHeaderBrand from '../header/NotebookHeaderBrand';
import { useLab, type LabRouteView } from '../../hooks/useLab';
import LabCompare from './LabCompare';
import LabDatasetList from './LabDatasetList';
import LabDatasetPreviewDialog from './LabDatasetPreviewDialog';
import LabJobForm from './LabJobForm';
import LabJobDetail from './LabJobDetail';
import LabJobList from './LabJobList';

type LabViewProps = {
  view: LabRouteView;
  onNavigate: (view: LabRouteView) => void;
  onHome?: () => void;
};

const LabView: FC<LabViewProps> = ({ view, onNavigate, onHome }) => {
  const lab = useLab();
  const [formDatasetId, setFormDatasetId] = useState<string | null>(null);
  const [compareMode, setCompareMode] = useState(false);
  const [compareIds, setCompareIds] = useState<Set<string>>(new Set());

  const toggleCompare = useCallback((jobId: string) => {
    setCompareIds((current) => {
      const next = new Set(current);
      if (next.has(jobId)) next.delete(jobId);
      else next.add(jobId);
      return next;
    });
  }, []);

  const pickDatasetFile = useCallback(async () => {
    const path = await openFileDialog({
      multiple: false,
      title: 'Upload dataset (CSV)',
      filters: [{ name: 'CSV', extensions: ['csv'] }],
    });
    if (typeof path === 'string') {
      const dataset = await lab.uploadDataset(path);
      if (dataset) setFormDatasetId(dataset.id);
    }
  }, [lab]);

  const downloadArtefact = useCallback(
    async (jobId: string, filename: string) => {
      const path = await saveFileDialog({
        title: 'Export model',
        defaultPath: filename,
      });
      if (typeof path === 'string') {
        await lab.exportJob(jobId, 'onnx', path);
      }
    },
    [lab],
  );

  return (
    <div data-component='LabView' className='flex h-full flex-col overflow-hidden'>
      <header
        data-component='LabHeader'
        className='flex min-h-14 shrink-0 items-center gap-4 border-b border-neutral-800 bg-neutral-900 px-4 py-2'>
        <NotebookHeaderBrand compact product='lab' onHome={onHome} />
        <span className='hidden text-xs text-neutral-500 sm:inline'>CSV → model → ONNX</span>
        <div className='ml-auto flex items-center gap-2'>
          {view.kind !== 'home' && (
            <Button size='sm' variant='ghost' onClick={() => onNavigate({ kind: 'home' })}>
              ← Back
            </Button>
          )}
          <Button
            size='sm'
            variant='primary'
            loading={lab.training}
            onClick={() => {
              setFormDatasetId(null);
              onNavigate({ kind: 'new-job' });
            }}>
            {lab.trainingStage ? `Training: ${lab.trainingStage}…` : 'New training job'}
          </Button>
        </div>
      </header>

      {lab.error && (
        <p className='shrink-0 border-b border-error/30 bg-error/10 px-4 py-2 text-xs text-error'>
          {lab.error}
        </p>
      )}

      <div className='min-h-0 flex-1 overflow-y-auto'>
        {lab.loading ? (
          <p className='p-6 text-sm text-neutral-500'>Loading Mars Lab…</p>
        ) : view.kind === 'new-job' ? (
          <LabJobForm
            datasets={lab.datasets}
            algorithms={lab.algorithms}
            training={lab.training}
            initialDatasetId={formDatasetId}
            onUploadDataset={() => void pickDatasetFile()}
            onTrain={(request) =>
              void lab.train(request).then((detail) => {
                if (detail) onNavigate({ kind: 'job', jobId: detail.job.id });
              })
            }
          />
        ) : view.kind === 'job' ? (
          <LabJobDetail
            detail={lab.selectedJob}
            onDownload={(jobId, filename) => void downloadArtefact(jobId, filename)}
          />
        ) : view.kind === 'compare' ? (
          <LabCompare details={lab.compareDetails} />
        ) : (
          <div className='grid gap-6 p-6 lg:grid-cols-2'>
            <section>
              <div className='mb-3 flex items-center justify-between'>
                <h2 className='text-sm font-medium text-neutral-300'>Datasets</h2>
                <Button size='sm' variant='secondary' onClick={() => void pickDatasetFile()}>
                  Upload CSV
                </Button>
              </div>
              <LabDatasetList
                datasets={lab.datasets}
                onSelect={(dataset) => void lab.showDatasetPreview(dataset)}
              />
            </section>
            <section>
              <div className='mb-3 flex flex-wrap items-center justify-between gap-2'>
                <h2 className='text-sm font-medium text-neutral-300'>Training jobs</h2>
                <div className='flex items-center gap-2'>
                  <select
                    aria-label='Filter by status'
                    className='h-7 rounded-md border border-neutral-700 bg-neutral-900 px-2 text-xs text-neutral-300'
                    value={lab.jobFilters.status}
                    onChange={(e) =>
                      lab.applyJobFilters({ ...lab.jobFilters, status: e.target.value })
                    }>
                    <option value=''>All statuses</option>
                    {['queued', 'validating', 'training', 'evaluating', 'exporting', 'completed', 'failed', 'cancelled'].map(
                      (s) => (
                        <option key={s} value={s}>
                          {s}
                        </option>
                      ),
                    )}
                  </select>
                  <select
                    aria-label='Filter by task'
                    className='h-7 rounded-md border border-neutral-700 bg-neutral-900 px-2 text-xs text-neutral-300'
                    value={lab.jobFilters.taskType}
                    onChange={(e) =>
                      lab.applyJobFilters({ ...lab.jobFilters, taskType: e.target.value })
                    }>
                    <option value=''>All tasks</option>
                    {[
                      'classification',
                      'regression',
                      'semi_supervised',
                      'clustering',
                      'anomaly_detection',
                      'reinforcement_learning',
                    ].map((t) => (
                      <option key={t} value={t}>
                        {t}
                      </option>
                    ))}
                  </select>
                  <input
                    aria-label='Search jobs'
                    placeholder='Search…'
                    className='h-7 w-28 rounded-md border border-neutral-700 bg-neutral-900 px-2 text-xs text-neutral-300'
                    value={lab.jobFilters.q}
                    onChange={(e) =>
                      lab.applyJobFilters({ ...lab.jobFilters, q: e.target.value })
                    }
                  />
                  {compareMode && compareIds.size >= 2 && (
                    <Button
                      size='sm'
                      variant='primary'
                      onClick={() => onNavigate({ kind: 'compare', jobIds: [...compareIds] })}>
                      Compare ({compareIds.size})
                    </Button>
                  )}
                  <Button
                    size='sm'
                    variant={compareMode ? 'primary' : 'ghost'}
                    onClick={() => {
                      setCompareMode((c) => !c);
                      setCompareIds(new Set());
                    }}>
                    Compare
                  </Button>
                </div>
              </div>
              <LabJobList
                jobs={lab.jobs}
                onSelect={(id) => onNavigate({ kind: 'job', jobId: id })}
                compareIds={compareMode ? compareIds : undefined}
                onToggleCompare={compareMode ? toggleCompare : undefined}
              />
              {lab.nextCursor && (
                <div className='mt-3 flex justify-center'>
                  <Button
                    size='sm'
                    variant='secondary'
                    loading={lab.loadingMore}
                    onClick={() => void lab.loadMoreJobs()}>
                    Load more
                  </Button>
                </div>
              )}
            </section>
          </div>
        )}
      </div>
      {lab.datasetPreview && (
        <LabDatasetPreviewDialog
          dataset={lab.datasetPreview.dataset}
          preview={lab.datasetPreview.preview}
          warnings={lab.datasetPreview.warnings}
          onClose={lab.closeDatasetPreview}
        />
      )}
    </div>
  );
};

LabView.displayName = 'LabView';
export default LabView;
