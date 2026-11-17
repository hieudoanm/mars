import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { readTablePreview } from '../../../lib/kernelService';
import { DataPreview } from '../DataPreview';

vi.mock('../../../lib/kernelService', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../../lib/kernelService')>();
  return { ...actual, readTablePreview: vi.fn() };
});

describe('DataPreview', () => {
  beforeEach(() => vi.clearAllMocks());

  it('does not load until the details element is opened', () => {
    render(<DataPreview path='/tmp/data.csv' />);
    expect(readTablePreview).not.toHaveBeenCalled();
    expect(screen.getByText('Data preview')).toBeInTheDocument();
  });

  it('loads and renders the table on toggle', async () => {
    vi.mocked(readTablePreview).mockResolvedValue({
      columns: ['a', 'b'],
      rows: [
        ['1', '2'],
        [null, '4'],
      ],
    });
    const { container } = render(<DataPreview path='/tmp/data.csv' />);
    const details = container.querySelector('details')!;
    fireEvent(details, new Event('toggle'));
    // jsdom does not flip `open` automatically; simulate the open state
    details.open = true;
    fireEvent(details, new Event('toggle'));

    await waitFor(() => expect(readTablePreview).toHaveBeenCalledWith('/tmp/data.csv'));
    await screen.findByText('a');
    expect(screen.getByText('b')).toBeInTheDocument();
  });

  it('shows an error when the preview fails', async () => {
    vi.mocked(readTablePreview).mockRejectedValue(new Error('no such file'));
    const { container } = render(<DataPreview path='/tmp/missing.csv' />);
    const details = container.querySelector('details')!;
    details.open = true;
    fireEvent(details, new Event('toggle'));
    await screen.findByText('no such file');
  });
});
