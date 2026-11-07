import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import TableOutput from '../TableOutput';

const columns = ['name', 'score'];
const rows = [
  ['bob', '20'],
  ['alice', '10'],
];

describe('TableOutput', () => {
  it('sorts rows when a column header is clicked', () => {
    render(<TableOutput columns={columns} rows={rows} />);
    const nameHeader = screen.getByRole('columnheader', { name: /name/ });
    fireEvent.click(nameHeader);
    const cells = screen.getAllByRole('cell').map((cell) => cell.textContent);
    expect(cells.indexOf('alice')).toBeLessThan(cells.indexOf('bob'));
    fireEvent.click(nameHeader);
    expect(cells.indexOf('alice')).toBeGreaterThan(-1);
  });

  it('copies the visible rows as CSV', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, { clipboard: { writeText } });
    render(<TableOutput columns={columns} rows={rows} />);
    fireEvent.click(screen.getByRole('button', { name: /Copy CSV/ }));
    await vi.waitFor(() => expect(writeText).toHaveBeenCalled());
    expect(writeText.mock.calls[0][0]).toContain('name,score');
    expect(writeText.mock.calls[0][0]).toContain('bob,20');
  });
});

describe('TableOutput filtering', () => {
  it('filters rows by cell content', () => {
    render(
      <TableOutput
        columns={['name', 'age']}
        rows={[
          ['ann', '30'],
          ['bob', '25'],
        ]}
      />,
    );
    fireEvent.change(screen.getByLabelText('Filter table rows'), { target: { value: 'ann' } });
    expect(screen.getByText('ann')).toBeInTheDocument();
    expect(screen.queryByText('bob')).not.toBeInTheDocument();
  });
});
