import * as fs from 'node:fs/promises';
import * as os from 'node:os';
import * as path from 'node:path';
import { describe, expect, it } from 'vitest';
import {
  checkpointsDirectory,
  createCheckpoint,
  listCheckpoints,
  readCheckpoint,
} from '../checkpoints';

const tempNotebook = async (): Promise<{ dir: string; notebook: string }> => {
  const dir = await fs.mkdtemp(path.join(os.tmpdir(), 'mars-cp-'));
  const notebook = path.join(dir, 'book.mars');
  await fs.writeFile(notebook, '{"cells":[]}');
  return { dir, notebook };
};

describe('checkpoints', () => {
  it('derives the checkpoint directory next to the notebook', () => {
    expect(checkpointsDirectory('/tmp/book.mars')).toBe('/tmp/book.mars.checkpoints');
    expect(checkpointsDirectory('/a/b/c.mars')).toBe(path.join('/a/b', 'c.mars.checkpoints'));
  });

  it('creates, lists, and reads checkpoints', async () => {
    const { notebook } = await tempNotebook();
    const first = await createCheckpoint(notebook);
    const second = await createCheckpoint(notebook);
    expect(first).not.toBe(second);
    expect(first).toContain('checkpoint-');
    expect(first.endsWith('.mars')).toBe(true);

    const listed = await listCheckpoints(notebook);
    expect(listed).toHaveLength(2);
    expect(listed[0]).toBe(first);
    expect(listed[1]).toBe(second);

    const content = await readCheckpoint(first);
    expect(content).toBe('{"cells":[]}');
  });

  it('lists nothing when the checkpoint directory is missing', async () => {
    const { notebook } = await tempNotebook();
    expect(await listCheckpoints(notebook)).toEqual([]);
  });

  it('ignores non-checkpoint files in the directory', async () => {
    const { notebook } = await tempNotebook();
    await createCheckpoint(notebook);
    const dir = checkpointsDirectory(notebook);
    await fs.writeFile(path.join(dir, 'notes.txt'), 'x');
    await fs.writeFile(path.join(dir, 'checkpoint-old.mars.bak'), 'x');
    await fs.writeFile(path.join(dir, 'other.mars'), 'x');
    const listed = await listCheckpoints(notebook);
    expect(listed).toHaveLength(1);
  });

  it('rejects checkpoint paths outside the checkpoint directory', async () => {
    await expect(readCheckpoint('/etc/passwd')).rejects.toThrow('outside the notebook');
  });
});
