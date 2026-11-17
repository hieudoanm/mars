import * as fs from 'node:fs/promises';
import * as path from 'node:path';

/**
 * Checkpoints mirror the desktop app's scheme: snapshots are stored in a
 * `<notebook>.checkpoints/` directory next to the notebook as
 * `checkpoint-<epoch-millis>.mars` files.
 */
export const checkpointsDirectory = (notebookPath: string): string => {
  return path.join(path.dirname(notebookPath), `${path.basename(notebookPath)}.checkpoints`);
};

export const createCheckpoint = async (notebookPath: string): Promise<string> => {
  const directory = checkpointsDirectory(notebookPath);
  await fs.mkdir(directory, { recursive: true });
  // Two checkpoints created within the same millisecond would collide, so
  // bump the timestamp until the target name is free.
  let timestamp = Date.now();
  let checkpoint = path.join(directory, `checkpoint-${timestamp}.mars`);
  while (
    await fs.stat(checkpoint).then(
      () => true,
      () => false,
    )
  ) {
    timestamp += 1;
    checkpoint = path.join(directory, `checkpoint-${timestamp}.mars`);
  }
  await fs.copyFile(notebookPath, checkpoint);
  return checkpoint;
};

export const listCheckpoints = async (notebookPath: string): Promise<string[]> => {
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
};

export const readCheckpoint = async (checkpointPath: string): Promise<string> => {
  const directory = path.dirname(checkpointPath);
  const resolved = path.resolve(checkpointPath);
  if (
    !directory.endsWith('.checkpoints') ||
    !resolved.startsWith(`${path.resolve(directory)}${path.sep}`)
  ) {
    throw new Error('Checkpoint is outside the notebook checkpoint directory');
  }
  return fs.readFile(resolved, 'utf-8');
};
