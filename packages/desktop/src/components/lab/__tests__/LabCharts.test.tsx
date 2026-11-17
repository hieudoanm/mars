import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { BarChart, Histogram, LineChart, ScatterPlot } from '../charts/primitives';

describe('LineChart', () => {
  it('returns null for mismatched or empty data', () => {
    const { container } = render(<LineChart x={[1, 2]} y={[1]} />);
    expect(container).toBeEmptyDOMElement();
    const empty = render(<LineChart x={[]} y={[]} />);
    expect(empty.container).toBeEmptyDOMElement();
  });

  it('renders a line chart with axes', () => {
    render(<LineChart x={[0, 1, 2]} y={[0, 1, 0]} xLabel='step' yLabel='loss' />);
    expect(screen.getByRole('img', { name: 'loss' })).toBeInTheDocument();
  });
});

describe('BarChart', () => {
  it('returns null for empty values', () => {
    const { container } = render(<BarChart labels={[]} values={[]} />);
    expect(container).toBeEmptyDOMElement();
  });

  it('renders vertical bars with labels', () => {
    render(<BarChart labels={['a', 'b']} values={[1, 2]} />);
    expect(screen.getByText('a')).toBeInTheDocument();
    expect(screen.getByText('b')).toBeInTheDocument();
  });

  it('renders horizontal bars', () => {
    render(<BarChart labels={['x', 'y']} values={[1, -2]} horizontal />);
    expect(screen.getByText('x')).toBeInTheDocument();
  });
});

describe('ScatterPlot', () => {
  it('returns null for empty data', () => {
    const { container } = render(<ScatterPlot x={[]} y={[]} />);
    expect(container).toBeEmptyDOMElement();
  });

  it('renders points with group and highlight colors', () => {
    render(
      <ScatterPlot x={[0, 1, 2]} y={[0, 1, 0]} groups={[0, 1]} highlight={[false, false, true]} />,
    );
    expect(screen.getByRole('img', { name: 'scatter plot' })).toBeInTheDocument();
  });
});

describe('Histogram', () => {
  it('returns null with too few edges', () => {
    const { container } = render(<Histogram edges={[0]} counts={[1]} />);
    expect(container).toBeEmptyDOMElement();
    const noCounts = render(<Histogram edges={[0, 1]} counts={[]} />);
    expect(noCounts.container).toBeEmptyDOMElement();
  });

  it('renders bars and a threshold line', () => {
    render(<Histogram edges={[0, 1, 2]} counts={[2, 5]} threshold={0.5} />);
    expect(screen.getByRole('img', { name: 'histogram' })).toBeInTheDocument();
  });
});
