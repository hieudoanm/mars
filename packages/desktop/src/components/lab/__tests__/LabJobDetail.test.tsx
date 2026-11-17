import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import LabJobDetail from '../LabJobDetail';
import type { LabJob, LabJobDetail as LabJobDetailData } from '../../../lib/kernelService';

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

const detail = (overrides: Partial<LabJobDetailData> = {}): LabJobDetailData => ({
  job: job(),
  metrics: null,
  artefacts: [],
  ...overrides,
});

describe('LabJobDetail', () => {
  it('shows a loading state without detail', () => {
    render(<LabJobDetail detail={null} onDownload={() => {}} />);
    expect(screen.getByText('Loading job…')).toBeInTheDocument();
  });

  it('renders job metadata and status badge', () => {
    render(<LabJobDetail detail={detail()} onDownload={() => {}} />);
    expect(screen.getAllByText('logistic_regression').length).toBeGreaterThan(0);
    expect(screen.getByText('completed')).toBeInTheDocument();
    expect(screen.getByText('classification')).toBeInTheDocument();
    expect(screen.getByText('a, b')).toBeInTheDocument();
    expect(screen.getByText('target')).toBeInTheDocument();
  });

  it('prefers the job name over the algorithm id', () => {
    render(
      <LabJobDetail detail={detail({ job: job({ name: 'My run' }) })} onDownload={() => {}} />,
    );
    expect(screen.getByText('My run')).toBeInTheDocument();
  });

  it('renders failure status and error message', () => {
    render(
      <LabJobDetail
        detail={detail({ job: job({ status: 'failed', errorMessage: 'oom' }) })}
        onDownload={() => {}}
      />,
    );
    expect(screen.getByText('failed')).toBeInTheDocument();
    expect(screen.getByText('oom')).toBeInTheDocument();
  });

  it('renders nested metrics', () => {
    render(
      <LabJobDetail
        detail={detail({
          metrics: {
            jobId: 'job-1',
            metrics: {
              accuracy: 0.95,
              nested: { loss: 0.1, flag: true },
              list: [1, 2],
            },
            charts: null,
            createdAt: '2026-01-01T00:00:00Z',
          },
        })}
        onDownload={() => {}}
      />,
    );
    expect(screen.getByText('accuracy')).toBeInTheDocument();
    expect(screen.getByText('0.9500')).toBeInTheDocument();
    expect(screen.getByText('nested.loss')).toBeInTheDocument();
    expect(screen.getByText('[2 items]')).toBeInTheDocument();
  });

  it('renders artefacts and triggers onnx download', () => {
    const onDownload = vi.fn();
    render(
      <LabJobDetail
        detail={detail({
          artefacts: [
            {
              id: 'a1',
              jobId: 'job-1',
              format: 'onnx',
              filename: 'model.onnx',
              byteSize: 4096,
              storageKey: 'k',
              createdAt: '2026-01-01T00:00:00Z',
            },
            {
              id: 'a2',
              jobId: 'job-1',
              format: 'csv',
              filename: 'predictions.csv',
              byteSize: 1024,
              storageKey: 'k2',
              createdAt: '2026-01-01T00:00:00Z',
            },
          ],
        })}
        onDownload={onDownload}
      />,
    );
    expect(screen.getByText('model.onnx')).toBeInTheDocument();
    expect(screen.getByText('predictions.csv')).toBeInTheDocument();
    // only the onnx artefact gets a download button
    const buttons = screen.getAllByRole('button', { name: 'Download ONNX' });
    expect(buttons).toHaveLength(1);
    fireEvent.click(buttons[0]);
    expect(onDownload).toHaveBeenCalledWith('job-1', 'model.onnx');
  });
});
