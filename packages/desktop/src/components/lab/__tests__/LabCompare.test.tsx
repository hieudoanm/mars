import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import LabCompare from '../LabCompare';
import type { LabJobDetail } from '../../../lib/kernelService';

const detail = (id: string, metrics: unknown, name: string | null = null): LabJobDetail =>
  ({
    job: {
      id,
      name,
      algorithmId: `algo-${id}`,
      status: 'completed',
    },
    metrics: metrics === null ? null : { metrics },
    artefacts: [],
  }) as unknown as LabJobDetail;

describe('LabCompare', () => {
  it('shows a loading state for no details', () => {
    render(<LabCompare details={[]} />);
    expect(screen.getByText('Loading jobs…')).toBeInTheDocument();
  });

  it('flattens nested metrics into comparison rows', () => {
    render(
      <LabCompare
        details={[
          detail(
            'j1',
            { accuracy: 0.95, nested: { loss: 0.1 }, tags: [1, 2], missing: null },
            'Run A',
          ),
          detail('j2', { accuracy: 0.8, nested: { loss: 0.2 } }, null),
        ]}
      />,
    );
    expect(screen.getByText('Metric')).toBeInTheDocument();
    expect(screen.getByText('Run A')).toBeInTheDocument();
    expect(screen.getByText('algo-j2')).toBeInTheDocument();
    expect(screen.getByText('accuracy')).toBeInTheDocument();
    expect(screen.getByText('nested.loss')).toBeInTheDocument();
    expect(screen.getByText('tags')).toBeInTheDocument();
    expect(screen.getByText('[2 items]')).toBeInTheDocument();
    // null renders as an em dash; j2 lacks `tags`/`missing` → em dash cells
    expect(screen.getAllByText('—').length).toBeGreaterThan(0);
  });

  it('shows an empty state when jobs have no metrics', () => {
    render(<LabCompare details={[detail('j1', null), detail('j2', null)]} />);
    expect(screen.getByText('Selected jobs have no metrics to compare.')).toBeInTheDocument();
  });
});
