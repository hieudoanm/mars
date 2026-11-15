import { EventEmitter } from 'node:events';
import type { ChildProcessWithoutNullStreams } from 'node:child_process';
import * as vscode from 'vscode';

const { spawn, commandExists } = vi.hoisted(() => ({ spawn: vi.fn(), commandExists: vi.fn() }));
vi.mock('node:child_process', () => ({ spawn }));
vi.mock('../cli', () => ({ commandExists, execFileAsync: vi.fn() }));

import { MarsKernelService } from '../kernelService';
import type { KernelRequest } from '../kernelProtocol';

function childProcess() {
  const child = new EventEmitter();
  const stdout = new EventEmitter();
  const writes: string[] = [];
  const stdin = {
    write: (value: string) => {
      writes.push(value);
      const request = JSON.parse(value) as KernelRequest;
      queueMicrotask(() =>
        stdout.emit(
          'data',
          Buffer.from(
            JSON.stringify({
              id: request.id,
              response:
                request.request.type === 'reset'
                  ? { type: 'reset', status: 'ready' }
                  : {
                      type: 'execution',
                      outputs: [{ type: 'text', text: '42' }],
                      execution_count: 2,
                      duration_millis: 1,
                    },
            }) + '\n',
          ),
        ),
      );
      return true;
    },
  };
  const kill = vi.fn();
  Object.assign(child, { stdout, stdin, kill });
  return {
    child: child as unknown as ChildProcessWithoutNullStreams,
    writes,
    kill,
  };
}

describe('MarsKernelService', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    spawn.mockReset();
    commandExists.mockReset().mockResolvedValue(true);
    vi.mocked(vscode.workspace.getConfiguration).mockReturnValue({
      get: (_key: string, fallback: unknown) => fallback,
    } as never);
  });
  afterEach(() => vi.useRealTimers());

  it('checks for the CLI, starts once, reuses the session, and restarts for another notebook', async () => {
    const processes = [childProcess(), childProcess()];
    spawn.mockReturnValueOnce(processes[0].child).mockReturnValueOnce(processes[1].child);
    const service = new MarsKernelService('mars-custom');
    const firstStart = service.start('/one.mars');
    await vi.runAllTimersAsync();
    await firstStart;
    await service.start('/one.mars');
    expect(spawn).toHaveBeenCalledTimes(1);
    const secondStart = service.start('/two.mars');
    await vi.runAllTimersAsync();
    await secondStart;
    expect(processes[0].kill).toHaveBeenCalledOnce();
    expect(spawn).toHaveBeenLastCalledWith(
      'mars-custom',
      ['kernel', 'serve', '--notebook', '/two.mars'],
      {
        stdio: ['pipe', 'pipe', 'pipe'],
      },
    );
    service.stop();
    expect(processes[1].kill).toHaveBeenCalledOnce();
  });

  it('adds the experimental runtime flag and reports a missing CLI', async () => {
    vi.mocked(vscode.workspace.getConfiguration).mockReturnValue({
      get: (key: string, fallback: unknown) => (key === 'runtime' ? 'evcxr' : fallback),
    } as never);
    const process = childProcess();
    spawn.mockReturnValue(process.child);
    const service = new MarsKernelService('mars');
    const start = service.start();
    await vi.runAllTimersAsync();
    await start;
    expect(spawn).toHaveBeenCalledWith(
      'mars',
      ['kernel', 'serve', '--runtime', 'evcxr'],
      expect.any(Object),
    );

    commandExists.mockResolvedValueOnce(false);
    await expect(new MarsKernelService('missing').start()).rejects.toThrow('Project Mars CLI not found at');
  });

  it('sends execution and reset requests and maps output failures', async () => {
    const process = childProcess();
    spawn.mockReturnValue(process.child);
    const service = new MarsKernelService('mars');
    const execution = service.execute('answer()', 1500, '/book.mars');
    await vi.runAllTimersAsync();
    await expect(execution).resolves.toEqual({
      status: 'success',
      execution_count: 2,
      outputs: [{ type: 'text', text: '42' }],
    });
    expect(JSON.parse(process.writes[0])).toMatchObject({
      request: { type: 'execute', source: 'answer()', timeout_millis: 1500 },
    });
    await expect(service.reset()).resolves.toBeUndefined();
    expect(JSON.parse(process.writes[1])).toMatchObject({ request: { type: 'reset' } });
  });

  it('reports a missing CLI after the child process reports an error', async () => {
    const process = childProcess();
    spawn.mockReturnValue(process.child);
    const service = new MarsKernelService('mars');
    const start = service.start();
    await vi.runAllTimersAsync();
    await start;
    (process.child as unknown as EventEmitter).emit('error', new Error('spawn failed'));
    commandExists.mockResolvedValueOnce(false);
    await expect(service.execute('1')).rejects.toThrow('Project Mars CLI not found at');
  });
});
