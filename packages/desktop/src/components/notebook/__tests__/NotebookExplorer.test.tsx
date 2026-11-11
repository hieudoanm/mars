import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { NotebookCell } from '../../../types/types';
import NotebookExplorer from '../NotebookExplorer';

const sidebarProps = vi.hoisted(() => ({ current: null as Record<string, unknown> | null }));

vi.mock('../NotebookSidebar', () => ({
  default: (props: Record<string, unknown>) => {
    sidebarProps.current = props;
    return (
      <div data-testid='sidebar' data-open={props.open as boolean}>
        <button type='button' onClick={() => (props.onSelect as (id: string) => void)('cell-1')}>
          pick-cell-1
        </button>
      </div>
    );
  },
}));

const cells: NotebookCell[] = [
  {
    id: 'cell-1',
    cell_type: 'code',
    source: 'fn main() {}',
    execution_count: null,
    outputs: [],
    metadata: {},
  },
];

type Props = Parameters<typeof NotebookExplorer>[0];

const makeProps = (overrides: Partial<Props> = {}): Props => ({
  open: true,
  cells,
  selectedId: 'cell-1',
  filePath: '/notes/a.mars',
  isDirty: false,
  onSelectCell: vi.fn(),
  onRenameFile: vi.fn(async () => true),
  onOpenSearch: vi.fn(),
  onOpenDependencies: vi.fn(),
  onOpenCheckpoints: vi.fn(),
  liveVariables: [],
  onRefreshVariables: vi.fn(),
  problems: [],
  onRevealLine: vi.fn(),
  setCommandMode: vi.fn(),
  ...overrides,
});

describe('NotebookExplorer', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    sidebarProps.current = null;
    window.requestAnimationFrame = (cb) => {
      cb(0);
      return 0;
    };
    document.body.innerHTML = '<div id="cell-cell-1"></div>' + document.body.innerHTML;
  });

  it('passes state through to the sidebar', () => {
    render(<NotebookExplorer {...makeProps()} />);
    expect(screen.getByTestId('sidebar')).toHaveAttribute('data-open', 'true');
    expect(sidebarProps.current?.filePath).toBe('/notes/a.mars');
    expect(sidebarProps.current?.isDirty).toBe(false);
  });

  it('selects a cell, switches to command mode, and scrolls it into view', () => {
    const props = makeProps();
    const scrollSpy = vi.fn();
    const el = document.getElementById('cell-cell-1')!;
    Object.defineProperty(el, 'scrollIntoView', { value: scrollSpy, configurable: true });
    render(<NotebookExplorer {...props} />);
    fireEvent.click(screen.getByText('pick-cell-1'));
    expect(props.onSelectCell).toHaveBeenCalledWith('cell-1');
    expect(props.setCommandMode).toHaveBeenCalled();
    expect(scrollSpy).toHaveBeenCalledWith({ block: 'center' });
  });

  it('maps heading reveals to line targets', () => {
    const props = makeProps();
    render(<NotebookExplorer {...props} />);
    (sidebarProps.current?.onRevealHeading as (cellId: string, line: number) => void)('cell-1', 4);
    expect(props.onRevealLine).toHaveBeenCalledWith({ cellId: 'cell-1', line: 4, character: 0 });
  });

  it('selects a problem cell and reveals its line', () => {
    const props = makeProps();
    render(<NotebookExplorer {...props} />);
    (sidebarProps.current?.onSelectProblem as (p: unknown) => void)({
      cellId: 'cell-1',
      cellIndex: 0,
      line: 7,
      severity: 'error',
      message: 'bad',
    });
    expect(props.onSelectCell).toHaveBeenCalledWith('cell-1');
    expect(props.setCommandMode).toHaveBeenCalled();
    expect(props.onRevealLine).toHaveBeenCalledWith({
      cellId: 'cell-1',
      line: 6,
      character: 0,
    });
  });
});
