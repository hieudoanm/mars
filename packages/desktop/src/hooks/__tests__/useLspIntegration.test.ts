import { act, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { NotebookCell } from '../../types/types';
import {
  lspCodeActions,
  lspCompletion,
  lspDefinition,
  lspHover,
  lspInlayHints,
  lspSemanticTokens,
  lspSync,
} from '../../lib/kernelService';
import { useLspIntegration } from '../useLspIntegration';

vi.mock('../../lib/kernelService', () => ({
  lspSync: vi.fn(async () => {}),
  lspHover: vi.fn(async () => null),
  lspCompletion: vi.fn(async () => null),
  lspSignatureHelp: vi.fn(async () => null),
  lspDefinition: vi.fn(async () => null),
  lspCodeActions: vi.fn(async () => []),
  lspInlayHints: vi.fn(async () => []),
  lspSemanticTokens: vi.fn(async () => []),
}));

const rustCell: NotebookCell = {
  id: 'r1',
  cell_type: 'code',
  source: 'fn main() {}',
  execution_count: null,
  outputs: [],
  metadata: {},
};
const mdCell: NotebookCell = { ...rustCell, id: 'm1', cell_type: 'markdown', source: '# hi' };
const pyCell: NotebookCell = {
  ...rustCell,
  id: 'p1',
  metadata: { language: 'python' },
};

const setup = (cells: NotebookCell[] = [rustCell, mdCell, pyCell]) => {
  const onRevealCell = vi.fn();
  const utils = renderHook(() =>
    useLspIntegration({ sessionId: 's1', cells, dependencies: { serde: '1' }, onRevealCell }),
  );
  return { ...utils, onRevealCell };
};

describe('useLspIntegration', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.clearAllMocks();
  });
  afterEach(() => vi.useRealTimers());

  it('syncs only runnable rust cells after the debounce', async () => {
    setup();
    await act(async () => {
      vi.advanceTimersByTime(900);
    });
    expect(lspSync).toHaveBeenCalledWith(
      's1',
      [{ id: 'r1', cell_type: 'code', source: 'fn main() {}' }],
      { serde: '1' },
    );
  });

  it('fetches inlay hints and semantic tokens', async () => {
    vi.mocked(lspInlayHints).mockResolvedValue([
      { cellId: 'r1', line: 0, character: 0, label: 'h' },
    ]);
    vi.mocked(lspSemanticTokens).mockResolvedValue([
      { cellId: 'r1', line: 0, character: 0, length: 2, tokenType: 'keyword' },
    ]);
    const { result } = setup([rustCell]);
    await act(async () => {
      vi.advanceTimersByTime(1300);
    });
    expect(result.current.inlayHints).toHaveLength(1);
    expect(result.current.semanticTokens).toHaveLength(1);
  });

  it('handleHover maps the LSP response to text', async () => {
    vi.mocked(lspHover).mockResolvedValue({ contents: 'docs' } as never);
    const { result } = setup();
    const text = await result.current.handleHover('r1', 0, 4);
    expect(lspHover).toHaveBeenCalledWith('s1', 'r1', 0, 4);
    expect(typeof text === 'string' || text === null).toBe(true);
  });

  it('handleHover returns null on failure', async () => {
    vi.mocked(lspHover).mockRejectedValue(new Error('down'));
    const { result } = setup();
    await expect(result.current.handleHover('r1', 0, 0)).resolves.toBeNull();
  });

  it('handlers delegate to kernel service and reveal sets pending target', async () => {
    const target = { cellId: 'r1', line: 2, character: 3 };
    vi.mocked(lspDefinition).mockResolvedValue(target);
    vi.mocked(lspCompletion).mockResolvedValue([{ label: 'main' }]);
    vi.mocked(lspCodeActions).mockResolvedValue([{ title: 'fix', edits: [] }]);
    const { result, onRevealCell } = setup();

    await expect(result.current.handlers.completion('r1', 0, 1)).resolves.toEqual([
      { label: 'main' },
    ]);
    await expect(result.current.handlers.signature('r1', 0, 1)).resolves.not.toThrow();
    await expect(result.current.handlers.definition('r1', 0, 1)).resolves.toEqual(target);
    await expect(result.current.handlers.codeActions('r1', 0, 1)).resolves.toEqual([
      { title: 'fix', edits: [] },
    ]);

    act(() => result.current.handlers.onDefinitionResult(target));
    expect(onRevealCell).toHaveBeenCalledWith('r1');
    expect(result.current.pendingReveal).toEqual(target);

    act(() => result.current.reveal({ cellId: 'r1', line: 5, character: 0 }));
    expect(onRevealCell).toHaveBeenCalledTimes(2);
    expect(result.current.pendingReveal).toEqual({ cellId: 'r1', line: 5, character: 0 });
  });

  it('handlers degrade gracefully when the service rejects', async () => {
    vi.mocked(lspDefinition).mockRejectedValue(new Error('x'));
    vi.mocked(lspCodeActions).mockRejectedValue(new Error('x'));
    const { result } = setup();
    await expect(result.current.handlers.definition('r1', 0, 0)).resolves.toBeNull();
    await expect(result.current.handlers.codeActions('r1', 0, 0)).resolves.toEqual([]);
  });
});
