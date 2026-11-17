import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import LabDatasetList from '../LabDatasetList';
import type { LabDataset } from '../../../lib/kernelService';

const dataset = (overrides: Partial<LabDataset> = {}): LabDataset => ({
  id: 'ds-1',
  originalFilename: 'data.csv',
  sha256: 'abc',
  byteSize: 2048,
  storageKey: 'key',
  columns: [
    { name: 'alpha', inferredDtype: 'f64', nullCount: 0, sampleValues: [] },
    { name: 'beta', inferredDtype: 'i64', nullCount: 0, sampleValues: [] },
  ],
  rowCount: 1500,
  createdAt: '2026-01-01T00:00:00Z',
  ...overrides,
});

describe('LabDatasetList', () => {
  it('shows the empty state', () => {
    render(<LabDatasetList datasets={[]} />);
    expect(screen.getByText(/No datasets yet/)).toBeInTheDocument();
  });

  it('renders dataset rows with shape and size', () => {
    render(<LabDatasetList datasets={[dataset()]} />);
    expect(screen.getByText('data.csv')).toBeInTheDocument();
    expect(screen.getByText('2 cols')).toBeInTheDocument();
    expect(screen.getByText('1,500 rows')).toBeInTheDocument();
    expect(screen.getByText('2.0 KB')).toBeInTheDocument();
    expect(screen.getByText('alpha, beta')).toBeInTheDocument();
  });

  it('formats byte sizes across units', () => {
    const { rerender } = render(<LabDatasetList datasets={[dataset({ byteSize: 500 })]} />);
    expect(screen.getByText('500 B')).toBeInTheDocument();
    rerender(<LabDatasetList datasets={[dataset({ byteSize: 5 * 1024 * 1024 })]} />);
    expect(screen.getByText('5.0 MB')).toBeInTheDocument();
  });

  it('calls onSelect with the dataset', () => {
    const onSelect = vi.fn();
    const ds = dataset();
    render(<LabDatasetList datasets={[ds]} onSelect={onSelect} />);
    fireEvent.click(screen.getByText('data.csv'));
    expect(onSelect).toHaveBeenCalledWith(ds);
  });
});
