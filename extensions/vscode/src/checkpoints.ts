import * as fs from 'node:fs/promises';
import * as path from 'node:path';

/**
 * Checkpoints mirror the desktop app's scheme: snapshots are stored in a
 * `<notebook>.checkpoints/` directory next to the notebook as
 * `checkpoint-<epoch-millis>.mars` files.
 */
export function checkpointsDirectory(notebookPath: string): string {
  return path.join(path.dirname(notebookPath), `${path.basename(notebookPath)}.checkpoints`);
}

export async function createCheckpoint(notebookPath: string): Promise<string> {
  const directory = checkpointsDirectory(notebookPath);
  await fs.mkdir(directory, { recursive: true });
  const checkpoint = path.join(directory, `checkpoint-${Date.now()}.mars`);
  await fs.copyFile(notebookPath, checkpoint);
  return checkpoint;
}

export async function listCheckpoints(notebookPath: string): Promise<string[]> {
  const directory = checkpointsDirectory(notebookPath);
  try {
    const entries = await fs.readdir(directory);
    return entries
      .filter((entry) => entry.startsWith('checkpoint-') && entry.endsWith('.mars'))
      .sort()
      .map((entry) => path.join(directory, entry));
  } catch {
    return [];
  }
}

export async function readCheckpoint(checkpointPath: string): Promise<string> {
  const directory = checkpointsDirectory(path.dirname(checkpointPath));
  const resolved = path.resolve(checkpointPath);
  if (!resolved.startsWith(`${path.resolve(directory)}${path.sep}`)) {
    throw new Error('Checkpoint is outside the notebook checkpoint directory');
  }
  return fs.readFile(resolved, 'utf-8');
}
