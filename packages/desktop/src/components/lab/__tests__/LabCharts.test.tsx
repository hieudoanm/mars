import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import LabCharts from '../LabCharts';
import LabCompare from '../LabCompare';
import type { LabJobDetail } from '../../../lib/kernelService';

describe('LabCharts', () => {
  it('renders nothing for empty charts', () => {
    const { container } = render(<LabCharts charts={null} />);
    expect(container).toBeEmptyDOMElement();
  });

  it('renders a confusion matrix and feature importance', () => {
    render(
      <LabCharts
        charts={{
          classes: ['a', 'b'],
          confusionMatrix: [
            [8, 2],
            [1, 9],
          ],
          featureImportance: {
            feature: ['f1', 'f2'],
            importance: [0.7, 0.3],
            method: 'weights',
          },
        }}
      />,
    );
    expect(screen.getByText('Confusion matrix')).toBeInTheDocument();
    expect(screen.getByText('true \\ pred')).toBeInTheDocument();
    expect(screen.getByText(/Feature importance/)).toBeInTheDocument();
  });

  it('renders mlp progress as a line chart', () => {
    const { container } = render(
      <LabCharts charts={{ mlpProgress: { epoch: [1, 2, 3], loss: [0.9, 0.5, 0.2] } }} />,
    );
    expect(screen.getByText('Training loss')).toBeInTheDocument();
    expect(container.querySelector('polyline')).not.toBeNull();
  });
});

describe('LabCompare', () => {
  const detail = (id: string, accuracy: number): LabJobDetail => ({
    job: {
      id,
      datasetId: 'ds',
      taskType: 'classification',
      algorithmId: `algo-${id}`,
      featureColumns: ['x'],
      labelColumn: 'y',
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
    },
    metrics: {
      jobId: id,
      metrics: { accuracy },
      charts: null,
      createdAt: '2026-01-01T00:00:00Z',
    },
    artefacts: [],
  });

  it('shows loading state with no details', () => {
    render(<LabCompare details={[]} />);
    expect(screen.getByText('Loading jobs…')).toBeInTheDocument();
  });

  it('compares metric values across jobs', () => {
    render(<LabCompare details={[detail('a', 0.9), detail('b', 0.8)]} />);
    expect(screen.getByText('accuracy')).toBeInTheDocument();
    expect(screen.getByText('0.900000')).toBeInTheDocument();
    expect(screen.getByText('0.800000')).toBeInTheDocument();
    expect(screen.getByText('algo-a')).toBeInTheDocument();
    expect(screen.getByText('algo-b')).toBeInTheDocument();
  });
});
