// Spawns `tauri-driver`, waits for its WebDriver port, runs the wdio suite,
// then shuts everything down. Used by `pnpm test:e2e:tauri`.
import { spawn } from 'node:child_process';
import net from 'node:net';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const here = path.dirname(fileURLToPath(import.meta.url));
const desktopDir = path.resolve(here, '..');
const port = Number(process.env.TAURI_DRIVER_PORT ?? 4444);

const waitForPort = (timeoutMs = 30_000) =>
  new Promise((resolve, reject) => {
    const deadline = Date.now() + timeoutMs;
    const attempt = () => {
      const socket = net.connect(port, '127.0.0.1');
      socket.once('connect', () => {
        socket.destroy();
        resolve();
      });
      socket.once('error', () => {
        socket.destroy();
        if (Date.now() > deadline) {
          reject(new Error(`tauri-driver did not listen on port ${port}`));
        } else {
          setTimeout(attempt, 250);
        }
      });
    };
    attempt();
  });

const driver = spawn('tauri-driver', [], {
  stdio: ['ignore', 'pipe', 'inherit'],
});
driver.stdout.on('data', (chunk) => process.stdout.write(`[tauri-driver] ${chunk}`));
driver.on('exit', (code) => {
  if (code !== null && code !== 0) {
    console.error(`tauri-driver exited early with code ${code}`);
  }
});

const wdio = spawn(process.platform === 'win32' ? 'wdio.cmd' : 'wdio', ['run', 'wdio.conf.ts'], {
  cwd: desktopDir,
  stdio: 'inherit',
  env: process.env,
});

const shutdown = () => {
  if (!driver.killed) driver.kill('SIGTERM');
};
process.on('SIGINT', () => {
  shutdown();
  process.exit(130);
});

try {
  await waitForPort();
  const code = await new Promise((resolve) => wdio.on('exit', resolve));
  shutdown();
  process.exit(code ?? 1);
} catch (error) {
  shutdown();
  console.error(error);
  process.exit(1);
}
