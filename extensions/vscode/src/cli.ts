import { execFile } from 'node:child_process';
import { promisify } from 'node:util';

export const execFileAsync = promisify(execFile);

export async function commandExists(
  command: string,
  runCommand: (file: string, args: string[]) => Promise<unknown> = execFileAsync,
): Promise<boolean> {
  const checkCommand = process.platform === 'win32' ? 'where' : 'which';
  try {
    await runCommand(checkCommand, [command]);
    return true;
  } catch {
    return false;
  }
}
