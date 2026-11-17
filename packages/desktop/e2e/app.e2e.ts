// True end-to-end tests: they drive the real Tauri app (built binary) through
// tauri-driver's WebDriver server, exercising the actual Rust backend — no
// mocks. Run with `pnpm test:e2e:tauri` (requires tauri-driver on PATH and a
// built `target/release/mars-tauri`, or set MARS_TAURI_APP).
//
// The specs share one app session and run sequentially: each builds on the
// notebook opened by the previous one.

// IconButton renders its label as `aria-label` (the visible text lives in a
// tooltip span), so match either button text or the aria-label attribute.
const button = (text: string) =>
  $(`//button[contains(normalize-space(.), "${text}") or @aria-label="${text}"]`);

// Some labels exist in more than one place (e.g. "New notebook" appears in
// both the welcome view and the file explorer); click the first match.
const clickButton = async (text: string) => {
  const buttons = await $$(
    `//button[contains(normalize-space(.), "${text}") or @aria-label="${text}"]`,
  );
  await buttons[0].waitForDisplayed({ timeout: 30_000 });
  await buttons[0].click();
};

const visible = async (selector: string) => {
  const el = $(selector);
  await el.waitForDisplayed({ timeout: 30_000 });
  return el;
};

const lastEditor = async () => {
  // Cells mount asynchronously after NotebookSession appears; poll for the
  // first editor before indexing into the list.
  await browser.waitUntil(async () => (await $$('.cm-content')).length > 0, {
    timeout: 30_000,
    interval: 500,
  });
  const editors = await $$('.cm-content');
  const editor = editors[editors.length - 1];
  await editor.waitForDisplayed({ timeout: 15_000 });
  return editor;
};

describe('Mars desktop (tauri-driver)', () => {
  it('shows the product welcome on first launch', async () => {
    await visible('[data-component="ProductWelcome"]');
    await button('Mars Notebook').waitForDisplayed({ timeout: 15_000 });
    await button('Mars Lab').waitForDisplayed();
  });

  it('opens the notebook workspace and a new notebook', async () => {
    // First launch routes to the product picker; the Mars Notebook card
    // enters the notebook workspace, which shows its own WelcomeView.
    await button('Mars Notebook').click();
    await visible('[data-component="WelcomeView"]');
    // "New notebook" also exists as an aria-label in the file explorer; scope
    // the click to the welcome view so we hit the session-creating button.
    const welcomeButtons = await $$(
      '[data-component="WelcomeView"] //button[contains(normalize-space(.), "New notebook")]',
    );
    await welcomeButtons[0].waitForDisplayed({ timeout: 15_000 });
    await welcomeButtons[0].click();
    await visible('[data-component="NotebookSession"]');
    await lastEditor();
  });

  it('adds a code cell and marks the notebook dirty', async () => {
    await clickButton('Add Code');
    await browser.waitUntil(
      async () => {
        const bars = await $$('footer, [data-component="NotebookStatusBar"]');
        for (const bar of bars) {
          try {
            if ((await bar.getText()).includes('Unsaved changes')) return true;
          } catch {
            // element detached mid-render; keep polling
          }
        }
        return false;
      },
      { timeout: 15_000, interval: 500 },
    );
  });

  it('executes a Rust cell through the real kernel', async () => {
    // "Add Code" selects the new cell and focuses its editor; type a cell that
    // prints a marker, then run it and wait for the captured output.
    const editor = await lastEditor();
    await editor.click();
    await editor.addValue('println!("mars-e2e-ok");');

    await clickButton('Run current cell');

    await browser.waitUntil(
      async () => {
        const outputs = await $$('[data-component="OutputView"]');
        for (const output of outputs) {
          try {
            if ((await output.getText()).includes('mars-e2e-ok')) return true;
          } catch {
            // element detached mid-render; keep polling
          }
        }
        return false;
      },
      // First Rust cell compile can be slow on a cold toolchain cache.
      { timeout: 300_000, interval: 2000 },
    );
  });
});
