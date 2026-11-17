import { describe, expect, it, vi } from 'vitest';
import { MarsVariablesProvider } from '../variablesView';
import type { MarsNotebookController } from '../notebookController';

const controllerWith = (inspect: () => Promise<unknown>) => {
  return { inspectVariables: vi.fn(inspect) } as unknown as MarsNotebookController;
};

describe('MarsVariablesProvider', () => {
  it('shows variables after a successful reload', async () => {
    const controller = controllerWith(async () => [
      { name: 'x', typeName: 'i32', value: '42' },
      { name: 'y' },
    ]);
    const provider = new MarsVariablesProvider(controller);
    await provider.reload();
    const children = provider.getChildren();
    expect(children).toHaveLength(2);
    expect(children[0].label).toBe('x');
    expect(children[0].description).toBe('i32');
    expect(children[1].tooltip).toBe('y');
    expect(provider.getTreeItem(children[0])).toBe(children[0]);
    provider.dispose();
    provider.refresh();
  });

  it('shows an error item when the kernel fails', async () => {
    const controller = controllerWith(async () => {
      throw new Error('kernel dead');
    });
    const provider = new MarsVariablesProvider(controller);
    await provider.reload();
    const children = provider.getChildren();
    expect(children).toHaveLength(1);
    expect(children[0].label).toContain('Kernel error: kernel dead');
    provider.dispose();
  });

  it('handles non-Error rejections and the empty state', async () => {
    const provider = new MarsVariablesProvider(
      controllerWith(async () => {
        throw 'string failure';
      }),
    );
    await provider.reload();
    expect(provider.getChildren()[0].label).toContain('string failure');

    const empty = new MarsVariablesProvider(controllerWith(async () => []));
    await empty.reload();
    expect(empty.getChildren()[0].label).toBe('No variables in session');
    expect(empty.getChildren(empty.getChildren()[0])).toEqual([]);
    empty.dispose();
  });
});
