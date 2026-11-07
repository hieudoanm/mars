import { expect, test } from '@playwright/test';

test('opens the notebook editor and exposes core workflows', async ({ page }) => {
  await page.goto('/');

  await expect(page.locator('.cm-content').first()).toContainText('Explore ideas');
  await expect(page.getByRole('button', { name: 'Run current cell' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Run all (F6)' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Add Code' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Add Markdown' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Delete selected cell (D, D)' })).toBeVisible();
  await expect(page.getByText('Hello from Mars!')).toBeVisible();
});

test('adds a code cell and marks the notebook dirty', async ({ page }) => {
  await page.goto('/');

  await page.getByRole('button', { name: 'Add Code' }).click();

  await expect(page.locator('footer').getByText('Unsaved changes')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Run all (F6)' })).toBeVisible();
});

test('persists editor preferences and supports undo/redo shortcuts', async ({ page }) => {
  await page.goto('/');

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
  await page.goto('/');

  const closeButtons = page.getByRole('button', { name: /^Close / });
  await expect(closeButtons).toHaveCount(1);

  await page.getByRole('button', { name: 'New notebook tab' }).click();
  await expect(closeButtons).toHaveCount(2);

  // allow the debounced untitled-notebook snapshot to flush
  await page.waitForTimeout(1200);
  await page.reload();
  await expect(closeButtons).toHaveCount(2);
});
