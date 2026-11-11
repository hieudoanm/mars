// Capture Mars desktop UI screenshots into assets/demo/png/.
//
// Driven by scripts/sh/screenshot.sh, which starts the Vite dev server first.
// Usage: node scripts/js/screenshot.mjs [baseUrl] [outDir]
// Defaults: http://127.0.0.1:4173 and <repo>/assets/demo/png
//
// Captures at a 1280x640 ratio (viewport 1280x640 CSS px at 2x DPR,
// producing 2560x1280 PNGs).

import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const desktop = path.join(root, 'packages', 'desktop');
const require = createRequire(path.join(desktop, 'package.json'));
const { chromium } = require('@playwright/test');

const base = process.argv[2] ?? 'http://127.0.0.1:4173';
const out = process.argv[3] ?? path.join(root, 'assets', 'demo', 'png');
const VIEW_W = 1280;
const VIEW_H = 640;
const DSF = 2;

const browser = await chromium.launch();
const page = await browser.newPage({
  viewport: { width: VIEW_W, height: VIEW_H },
  deviceScaleFactor: DSF,
});
await page.goto(`${base}/`);
await page.waitForTimeout(800);

const welcome = page.locator('[data-component="WelcomeView"]');
if (await welcome.isVisible()) {
  await page.screenshot({ path: path.join(out, 'desktop-welcome.png') });
  await welcome.getByRole('button', { name: 'New notebook' }).click();
}
await page.waitForTimeout(1200);
await page.screenshot({ path: path.join(out, 'desktop-overview.png') });
await browser.close();
console.log('screenshots written to', out);
