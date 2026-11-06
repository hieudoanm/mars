import { commandExists, execFileAsync } from '../cli';

describe('CLI discovery helpers', () => {
  it('runs a platform-aware command lookup', async () => {
    const runCommand = vi.fn().mockResolvedValue(undefined);
    await expect(commandExists('mars', runCommand)).resolves.toBe(true);
    expect(runCommand).toHaveBeenCalledWith(process.platform === 'win32' ? 'where' : 'which', ['mars']);
  });

  it('returns false when the command cannot be resolved', async () => {
    const runCommand = vi.fn().mockRejectedValue(new Error('not found'));
    await expect(commandExists('mars', runCommand)).resolves.toBe(false);
  });

  it('exports the promisified process helper', () => {
    expect(execFileAsync).toBeTypeOf('function');
  });
});
