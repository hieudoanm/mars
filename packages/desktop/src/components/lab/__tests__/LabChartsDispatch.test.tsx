import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import LabCharts from '../LabCharts';

describe('LabCharts', () => {
  it('renders nothing without charts', () => {
    const { container } = render(<LabCharts charts={null} />);
    expect(container).toBeEmptyDOMElement();
    const empty = render(<LabCharts charts={{}} />);
    expect(empty.container).toBeEmptyDOMElement();
  });

  it('renders a confusion matrix with class labels', () => {
    render(
      <LabCharts
        charts={{
          classes: ['cat', 'dog'],
          confusionMatrix: [
            [3, 1],
            [0, 4],
          ],
        }}
      />,
    );
    expect(screen.getByText('Confusion matrix')).toBeInTheDocument();
    expect(screen.getAllByText('cat').length).toBeGreaterThan(0);
    expect(screen.getAllByText('dog').length).toBeGreaterThan(0);
  });

  it('renders feature importance with a method label', () => {
    render(
      <LabCharts
        charts={{
          featureImportance: {
            method: 'gain',
            feature: ['age', 'income'],
            importance: [0.6, 0.4],
          },
        }}
      />,
    );
    expect(screen.getByText(/Feature importance \(gain\)/)).toBeInTheDocument();
    expect(screen.getByText('age')).toBeInTheDocument();
  });

  it('ignores malformed chart payloads', () => {
    render(
      <LabCharts
        charts={{
          featureImportance: 'nope',
          mlpProgress: { epoch: [1], loss: 'x' },
          projection: { x: [1], y: [2] },
          regressionPoints: { predicted: [1], actual: 'x' },
          residuals: null,
          clusterSizes: 'nope',
          scoreHistogram: { edges: 'x', counts: [] },
          qValueHistogram: null,
          tdErrorHistogram: {},
          scorePoints: { score: [1], anomaly: [true] },
          fqiProgress: [{ iteration: 1, meanTarget: 'x' }],
        }}
      />,
    );
    // Only the well-formed scorePoints card should render
    expect(screen.getByText('Anomaly scores')).toBeInTheDocument();
    expect(screen.queryByText('Feature importance')).not.toBeInTheDocument();
    expect(screen.queryByText('Training loss')).not.toBeInTheDocument();
  });

  it('renders the remaining chart cards', () => {
    render(
      <LabCharts
        charts={{
          mlpProgress: { epoch: [1, 2], loss: [0.5, 0.2] },
          regressionPoints: { predicted: [1, 2], actual: [1, 3] },
          residuals: { predicted: [1, 2], residual: [0.1, -0.2] },
          projection: { x: [1, 2, 3], y: [1, 2, 1], cluster: [0, 1, 0] },
          clusterSizes: [5, 3],
          scoreHistogram: { edges: [0, 1, 2], counts: [3, 4], threshold: 0.8 },
          qValueHistogram: { edges: [0, 1, 2], counts: [1, 2] },
          tdErrorHistogram: { edges: [0, 1, 2], counts: [1, 1] },
          scorePoints: {
            score: [0.1, 0.9],
            anomaly: [false, true],
            index: [10, 20],
          },
          fqiProgress: [
            { iteration: 1, meanTarget: 0.5 },
            { iteration: 2, meanTarget: 0.4 },
          ],
        }}
      />,
    );
    expect(screen.getByText('Training loss')).toBeInTheDocument();
    expect(screen.getByText('Predicted vs actual')).toBeInTheDocument();
    expect(screen.getByText('Residuals')).toBeInTheDocument();
    expect(screen.getByText('Cluster projection (PCA)')).toBeInTheDocument();
    expect(screen.getByText('Cluster sizes')).toBeInTheDocument();
    expect(screen.getByText('Score histogram')).toBeInTheDocument();
    expect(screen.getByText('Q-value histogram')).toBeInTheDocument();
    expect(screen.getByText('TD-error histogram')).toBeInTheDocument();
    expect(screen.getByText('Anomaly scores')).toBeInTheDocument();
    expect(screen.getByText('FQI progress')).toBeInTheDocument();
  });

  it('renders an empty confusion matrix as nothing inside its card', () => {
    render(<LabCharts charts={{ classes: ['a'], confusionMatrix: [] }} />);
    expect(screen.getByText('Confusion matrix')).toBeInTheDocument();
    expect(screen.queryByText('a')).not.toBeInTheDocument();
  });
});
