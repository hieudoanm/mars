import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Path to the compiled Tauri binary under test. Override with
// MARS_TAURI_APP when running against a different build (e.g. debug).
const application =
  process.env.MARS_TAURI_APP ?? path.resolve(__dirname, '../../target/release/mars-tauri');

export const config: WebdriverIO.Config = {
  runner: 'local',
  specs: ['./e2e/**/*.e2e.ts'],
  exclude: [],
  maxInstances: 1,
  capabilities: [
    {
      // No `browserName`: on Linux tauri-driver rewrites `tauri:options` into
      // `webkitgtk:browserOptions` for capability matching, and a client-side
      // browserName makes WebKitWebDriver reject the session. The `wry` label
      // is assigned by the server after the session is created.
      'tauri:options': {
        application,
      },
    } as WebdriverIO.Capabilities,
  ],
  // tauri-driver exposes a W3C WebDriver server; wdio connects to it directly.
  hostname: '127.0.0.1',
  port: 4444,
  path: '/',
  logLevel: 'warn',
  outputDir: './e2e/artifacts',
  waitforTimeout: 10_000,
  connectionRetryTimeout: 120_000,
  connectionRetryCount: 3,
  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: {
    ui: 'bdd',
    timeout: 600_000,
  },
};
