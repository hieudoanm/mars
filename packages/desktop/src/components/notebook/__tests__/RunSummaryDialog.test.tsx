import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import RunSummaryDialog from '../RunSummaryDialog';
import type { NotebookCell } from '../../../types/types';
import type { RunSummary } from '../../../hooks/useKernelExecution';

const cell = (id: string, source: string): NotebookCell => ({
  id,
  cell_type: 'code',
  source,
  outputs: [],
  metadata: {},
  execution_count: null,
});

const cells = [cell('c1', 'let x = 1;'), cell('c2', '// a comment\nprintln!("hi");')];

const summary = (entries: RunSummary['entries']): RunSummary => ({
  entries,
  executionMs: 1500,
  wallMs: 2300,
  finishedAt: Date.now(),
});

const entry = (
  cellId: string,
  status: 'success' | 'error' | 'interrupted',
  durationMs: number,
) => ({ cellId, status, durationMs });

describe('RunSummaryDialog', () => {
  it('renders nothing when closed or missing a summary', () => {
    const { container } = render(
      <RunSummaryDialog
        open={false}
        summary={summary([entry('c1', 'success', 10)])}
        cells={cells}
        notebookLanguage='rust'
        onClose={() => {}}
      />,
    );
    expect(container).toBeEmptyDOMElement();
    const noSummary = render(
      <RunSummaryDialog
        open
        summary={null}
        cells={cells}
        notebookLanguage='rust'
        onClose={() => {}}
      />,
    );
    expect(noSummary.container).toBeEmptyDOMElement();
  });

  it('lists entries slowest-first with status labels and counts', () => {
    render(
      <RunSummaryDialog
        open
        summary={summary([
          entry('c1', 'success', 10),
          entry('c2', 'error', 900),
          entry('ghost', 'interrupted', 100),
        ])}
        cells={cells}
        notebookLanguage='rust'
        onClose={() => {}}
      />,
    );
    expect(screen.getByText('Run summary')).toBeInTheDocument();
    expect(screen.getByText(/3 cells · 1 succeeded/)).toBeInTheDocument();
    expect(screen.getByText(/1 failed/)).toBeInTheDocument();
    expect(screen.getByText(/1 interrupted/)).toBeInTheDocument();
    expect(screen.getByText('Slowest cell:')).toBeInTheDocument();
    expect(screen.getByText('failed')).toBeInTheDocument();
    expect(screen.getByText('interrupted')).toBeInTheDocument();
    // Unknown cell falls back to its id and '?'
    expect(screen.getByText('ghost')).toBeInTheDocument();
  });

  it('previews the first non-empty source line, truncated at 60 chars', () => {
    const long = 'x'.repeat(80);
    render(
      <RunSummaryDialog
        open
        summary={summary([entry('c1', 'success', 5)])}
        cells={[cell('c1', `\n\n${long}`)]}
        notebookLanguage='rust'
        onClose={() => {}}
      />,
    );
    expect(screen.getByText(`${'x'.repeat(57)}…`)).toBeInTheDocument();
  });

  it('hides the slowest-cell line for a single entry', () => {
    render(
      <RunSummaryDialog
        open
        summary={summary([entry('c1', 'success', 5)])}
        cells={cells}
        notebookLanguage='rust'
        onClose={() => {}}
      />,
    );
    expect(screen.queryByText('Slowest cell:')).not.toBeInTheDocument();
  });

  it('closes via the close button', () => {
    const onClose = vi.fn();
    render(
      <RunSummaryDialog
        open
        summary={summary([entry('c1', 'success', 5)])}
        cells={cells}
        notebookLanguage='rust'
        onClose={onClose}
      />,
    );
    fireEvent.click(screen.getByLabelText('Close'));
    expect(onClose).toHaveBeenCalled();
  });
});
