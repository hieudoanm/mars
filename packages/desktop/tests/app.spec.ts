import { expect, test, type Page } from '@playwright/test';

const startNotebook = async (page: Page) => {
  await page.goto('/');
  const welcome = page.locator('[data-component="WelcomeView"]');
  if (await welcome.isVisible()) {
    await welcome.getByRole('button', { name: 'New notebook' }).click();
  }
};

test('shows the welcome view on first launch', async ({ page }) => {
  await page.goto('/');
  const welcome = page.locator('[data-component="WelcomeView"]');
  await expect(welcome).toBeVisible();
  await expect(welcome.getByRole('button', { name: 'Open notebook…' })).toBeVisible();
  await expect(welcome.getByRole('button', { name: 'Open folder…' })).toBeVisible();
  await expect(welcome.getByRole('button', { name: 'New notebook' })).toBeVisible();
});

test('opens the notebook editor and exposes core workflows', async ({ page }) => {
  await startNotebook(page);

  await expect(
    page.getByRole('heading', { name: 'Getting started with Mars' }).first(),
  ).toBeVisible();
  await expect(page.getByRole('button', { name: 'Run current cell' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Run all (F6)' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Add Code' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Add Markdown' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Delete selected cell (D, D)' })).toBeVisible();
  await expect(page.getByText('How a notebook works').first()).toBeVisible();
});

test('adds a code cell and marks the notebook dirty', async ({ page }) => {
  await startNotebook(page);

  await page.getByRole('button', { name: 'Add Code' }).click();

  await expect(page.locator('footer').getByText('Unsaved changes')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Run all (F6)' })).toBeVisible();
});

test('persists editor preferences and supports undo/redo shortcuts', async ({ page }) => {
  await startNotebook(page);

  const editor = page.locator('.cm-content').first();
  await editor.click();
  await page.keyboard.press('Control+A');
  await page.keyboard.type('let answer = 42;');
  await expect(editor).toContainText('let answer = 42;');

  await page.keyboard.press('Control+Z');
  await expect(editor).not.toContainText('let answer = 42;');
  await page.keyboard.press('Control+Shift+Z');
  await expect(editor).toContainText('let answer = 42;');

  await page.getByLabel('Editor font size').selectOption('18');
  await expect(page.getByLabel('Editor font size')).toHaveValue('18');

  await page.reload();
  await expect(page.getByLabel('Editor font size')).toHaveValue('18');
});

test('persists open notebook tabs across reload', async ({ page }) => {
  await startNotebook(page);

  const closeButtons = page.getByRole('button', { name: /^Close / });
  await expect(closeButtons).toHaveCount(1);

  await page.getByRole('button', { name: 'New notebook tab' }).click();
  await expect(closeButtons).toHaveCount(2);

  // allow the debounced untitled-notebook snapshot to flush
  await page.waitForTimeout(1200);
  await page.reload();
  await expect(closeButtons).toHaveCount(2);
});

test('global search reveals the exact match in the target cell', async ({ page }) => {
  await startNotebook(page);

  await page.getByRole('button', { name: 'Search notebook' }).click();
  const dialog = page.getByRole('dialog', { name: 'Search notebook' });
  await expect(dialog).toBeVisible();

  await page.getByLabel('Search query').fill('struct Report');
  const match = dialog.getByRole('button', { name: /Cell 6 · code/ });
  await expect(match).toBeVisible();
  await match.click();

  await expect(dialog).not.toBeVisible();
  // The reveal path selects the target cell and scrolls it into view: the
  // code cell containing the match must carry the selected-cell styling.
  const targetCell = page.locator('.border-orange-800', { hasText: 'struct Report' });
  await expect(targetCell).toBeVisible();
});

test('question mark opens settings in command mode', async ({ page }) => {
  await startNotebook(page);
  // Enter command mode with focus outside the editor, then use the
  // shortcuts-help key (command keys are ignored while a cell editor owns
  // focus, matching the other single-key commands).
  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
  await page.keyboard.press('Escape');
  await page.keyboard.press('?');
  await expect(page.getByRole('dialog', { name: 'Settings' })).toBeVisible();
});
