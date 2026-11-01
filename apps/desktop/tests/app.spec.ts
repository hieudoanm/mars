import { expect, test } from "@playwright/test";

test("opens the notebook editor and exposes core workflows", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByRole("heading", { name: "Explore ideas." })).toBeVisible();
  await expect(page.getByRole("button", { name: "Run all" })).toBeVisible();
  await expect(page.getByRole("button", { name: "+ Code" })).toBeVisible();
  await expect(page.getByRole("button", { name: "+ Markdown" })).toBeVisible();
  await expect(page.getByText("Hello from Mars!")).toBeVisible();
});

test("adds a code cell and marks the notebook dirty", async ({ page }) => {
  await page.goto("/");

  await page.getByRole("button", { name: "+ Code" }).click();

  await expect(page.locator("footer").getByText("Unsaved changes")).toBeVisible();
  await expect(page.getByRole("button", { name: "Run all" })).toBeVisible();
});

test("persists editor preferences and supports undo/redo shortcuts", async ({ page }) => {
  await page.goto("/");

  const editor = page.locator(".cm-content").first();
  await editor.click();
  await page.keyboard.press("Control+A");
  await page.keyboard.type("let answer = 42;");
  await expect(editor).toContainText("let answer = 42;");

  await page.keyboard.press("Control+Z");
  await expect(editor).not.toContainText("let answer = 42;");
  await page.keyboard.press("Control+Shift+Z");
  await expect(editor).toContainText("let answer = 42;");

  await page.getByLabel("Editor font size").fill("18");
  await page.getByLabel("Autocomplete").uncheck();
  await page.getByLabel("Folding").uncheck();
  await expect(page.getByLabel("Editor font size")).toHaveValue("18");
  await expect(page.getByLabel("Autocomplete")).not.toBeChecked();
  await expect(page.getByLabel("Folding")).not.toBeChecked();

  await page.reload();
  await expect(page.getByLabel("Editor font size")).toHaveValue("18");
  await expect(page.getByLabel("Autocomplete")).not.toBeChecked();
  await expect(page.getByLabel("Folding")).not.toBeChecked();
});
