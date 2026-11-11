import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { NotebookProblems } from '../NotebookProblems';
import type { NotebookProblem } from '../NotebookSidebar';

const problems: NotebookProblem[] = [
  { cellId: 'a', cellIndex: 0, line: 3, severity: 'error', message: 'type mismatch' },
  { cellId: 'b', cellIndex: 1, line: 7, severity: 'warning', message: 'unused var' },
  { cellId: 'c', cellIndex: 2, line: 1, severity: 'error', message: 'missing fn' },
];

const setup = (overrides: Partial<Parameters<typeof NotebookProblems>[0]> = {}) => {
  const onSelectCell = vi.fn();
  const onSelectProblem = vi.fn();
  render(<NotebookProblems problems={problems} onSelectCell={onSelectCell} {...overrides} />);
  return { onSelectCell, onSelectProblem };
};

describe('NotebookProblems', () => {
  beforeEach(() => {
    Object.defineProperty(navigator, 'clipboard', {
      value: { writeText: vi.fn(async () => {}) },
      configurable: true,
    });
  });

  it('shows an empty state without problems', () => {
    render(<NotebookProblems problems={[]} onSelectCell={vi.fn()} />);
    expect(screen.getByText('No problems')).toBeInTheDocument();
  });

  it('lists problems and selects one', () => {
    const { onSelectCell } = setup();
    fireEvent.click(screen.getByRole('button', { name: /type mismatch/ }));
    expect(onSelectCell).toHaveBeenCalledWith('a');
  });

  it('prefers onSelectProblem when provided', () => {
    const onSelectProblem = vi.fn();
    const { onSelectCell } = setup({ onSelectProblem });
    fireEvent.click(screen.getByRole('button', { name: /unused var/ }));
    expect(onSelectProblem).toHaveBeenCalledWith(problems[1]);
    expect(onSelectCell).not.toHaveBeenCalled();
  });

  it('filters by severity', () => {
    setup();
    expect(screen.getAllByRole('listitem')).toHaveLength(3);
    fireEvent.click(screen.getByRole('button', { name: 'error' }));
    expect(screen.getAllByRole('listitem')).toHaveLength(2);
    expect(screen.queryByText('unused var')).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'warning' }));
    expect(screen.getAllByRole('listitem')).toHaveLength(1);
    fireEvent.click(screen.getByRole('button', { name: 'all' }));
    expect(screen.getAllByRole('listitem')).toHaveLength(3);
  });

  it('cycles problems with prev/next buttons', () => {
    const { onSelectCell } = setup();
    fireEvent.click(screen.getByRole('button', { name: 'Next problem' }));
    expect(onSelectCell).toHaveBeenLastCalledWith('a');
    fireEvent.click(screen.getByRole('button', { name: 'Next problem' }));
    expect(onSelectCell).toHaveBeenLastCalledWith('b');
    fireEvent.click(screen.getByRole('button', { name: 'Previous problem' }));
    expect(onSelectCell).toHaveBeenLastCalledWith('a');
    // wrap-around backwards
    fireEvent.click(screen.getByRole('button', { name: 'Previous problem' }));
    expect(onSelectCell).toHaveBeenLastCalledWith('c');
  });

  it('copies all problems to the clipboard', async () => {
    const writeText = vi.fn<(text: string) => Promise<void>>(async () => {});
    Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true });
    setup();
    fireEvent.click(screen.getByRole('button', { name: 'Copy all' }));
    await vi.waitFor(() => expect(writeText).toHaveBeenCalled());
    expect(writeText.mock.calls[0]?.[0]).toContain('Cell 1: type mismatch');
    expect(writeText.mock.calls[0]?.[0]).toContain('Cell 3: missing fn');
    expect(await screen.findByRole('button', { name: 'Copied' })).toBeInTheDocument();
  });
});
