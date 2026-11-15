// Capture Project Mars desktop UI screenshots into assets/demo/png/.
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
const productWelcome = page.locator('[data-component="ProductWelcome"]');

// Capture product selection screen (Notebook vs Lab)
if (await productWelcome.isVisible()) {
  await page.screenshot({ path: path.join(out, 'desktop-welcome.png') });

  // Navigate to Lab
  await page.getByRole('button', { name: /mars lab/i }).click();
  await page.waitForTimeout(1200);
  await page.screenshot({ path: path.join(out, 'desktop-lab.png') });

  // Navigate to new job form
  const newJobButton = page.getByRole('button', { name: /new training job/i });
  if (await newJobButton.isVisible()) {
    await newJobButton.click();
    await page.waitForTimeout(800);
    await page.screenshot({ path: path.join(out, 'desktop-lab-new-job.png') });
  }

  // Go back to lab home, then to product selection
  const backToHome = page.getByRole('button', { name: '← Back' });
  if (await backToHome.isVisible()) {
    await backToHome.click();
    await page.waitForTimeout(600);
  }

  const backToProducts = page.getByRole('button', { name: 'Back to product selection' });
  if (await backToProducts.isVisible()) {
    await backToProducts.click();
    await page.waitForTimeout(600);
  }

  // Navigate to Notebook
  await page.getByRole('button', { name: /mars notebook/i }).click();
  await page.waitForTimeout(1200);
} else if (await welcome.isVisible()) {
  // Legacy welcome screen
  await page.screenshot({ path: path.join(out, 'desktop-welcome.png') });
  await welcome.getByRole('button', { name: 'New notebook' }).click();
  await page.waitForTimeout(1200);
} else {
  // Already on notebook view
  await page.waitForTimeout(600);
}

// If we're on notebook view, capture notebook screens
const notebookView = page.locator('[data-component="App"]');
if (await notebookView.isVisible()) {
  await page.waitForTimeout(600);

  // Capture notebook with cells
  await page.screenshot({ path: path.join(out, 'desktop-notebook.png') });

  // Try to open file explorer if available
  const explorerButton = page.getByRole('button', { name: /explorer|files/i });
  if (await explorerButton.isVisible()) {
    await explorerButton.click();
    await page.waitForTimeout(500);
    await page.screenshot({ path: path.join(out, 'desktop-notebook-explorer.png') });
  }
}

await browser.close();
console.log('screenshots written to', out);
