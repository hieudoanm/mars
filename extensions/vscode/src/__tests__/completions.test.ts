import * as vscode from 'vscode';
import { MarsCompletionProvider } from '../completions';
import type { MarsNotebookController } from '../notebookController';

const cellDocument = (overrides: Partial<vscode.TextDocument> = {}) =>
  ({
    uri: { scheme: 'vscode-notebook-cell' },
    languageId: 'rust',
    ...overrides,
  }) as vscode.TextDocument;

describe('MarsCompletionProvider', () => {
  it('ignores non-notebook and non-rust documents', async () => {
    const provider = new MarsCompletionProvider(() => null);
    await expect(
      provider.provideCompletionItems(cellDocument({ uri: { scheme: 'file' } as never })),
    ).resolves.toEqual([]);
    await expect(
      provider.provideCompletionItems(cellDocument({ languageId: 'markdown' })),
    ).resolves.toEqual([]);
  });

  it('suggests live kernel variables before keywords', async () => {
    const controller = {
      kernelRunning: () => true,
      inspectVariables: vi.fn().mockResolvedValue([
        { name: 'numbers', typeName: 'Vec<i32>', value: '[1, 2, 3]' },
        { name: 'total', typeName: null, value: null },
      ]),
    } as unknown as MarsNotebookController;
    const provider = new MarsCompletionProvider(() => controller);
    const items = await provider.provideCompletionItems(cellDocument());
    const names = items.map((item) => item.label);
    expect(names).toContain('numbers');
    expect(names).toContain('total');
    expect(names).toContain('let');
    expect(names.indexOf('numbers')).toBeLessThan(names.indexOf('let'));
    const numbers = items.find((item) => item.label === 'numbers');
    expect(numbers?.detail).toBe('Vec<i32>');
    expect(controller.inspectVariables).toHaveBeenCalledTimes(1);
  });

  it('does not spawn a kernel for completions', async () => {
    const controller = {
      kernelRunning: () => false,
      inspectVariables: vi.fn(),
    } as unknown as MarsNotebookController;
    const provider = new MarsCompletionProvider(() => controller);
    const items = await provider.provideCompletionItems(cellDocument());
    expect(items.map((item) => item.label)).toContain('fn');
    expect(controller.inspectVariables).not.toHaveBeenCalled();
  });

  it('keeps the last variable snapshot when the kernel is unreachable', async () => {
    const controller = {
      kernelRunning: () => true,
      inspectVariables: vi.fn().mockRejectedValue(new Error('kernel gone')),
    } as unknown as MarsNotebookController;
    const provider = new MarsCompletionProvider(() => controller);
    const items = await provider.provideCompletionItems(cellDocument());
    expect(items.map((item) => item.label)).toContain('match');
  });
});
