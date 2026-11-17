// True end-to-end tests: they drive the real Tauri app (built binary) through
// tauri-driver's WebDriver server, exercising the actual Rust backend — no
// mocks. Run with `pnpm test:e2e:tauri` (requires tauri-driver on PATH and a
// built `target/release/mars-tauri`, or set MARS_TAURI_APP).
//
// The specs share one app session and run sequentially: each builds on the
// notebook opened by the previous one.

const button = (text: string) => $(`//button[contains(normalize-space(.), "${text}")]`);

const visible = async (selector: string) => {
  const el = $(selector);
  await el.waitForDisplayed({ timeout: 30_000 });
  return el;
};

const lastEditor = async () => {
  const editors = await $$('.cm-content');
  const editor = editors[editors.length - 1];
  await editor.waitForDisplayed({ timeout: 15_000 });
  return editor;
};

describe('Mars desktop (tauri-driver)', () => {
  it('shows the welcome view on first launch', async () => {
    await visible('[data-component="WelcomeView"]');
    await button('New notebook').waitForDisplayed({ timeout: 15_000 });
    await button('Open notebook…').waitForDisplayed();
  });

  it('opens a notebook from the welcome view', async () => {
    await button('New notebook').click();
    await visible('[data-component="NotebookSession"]');
    await lastEditor();
  });

  it('adds a code cell and marks the notebook dirty', async () => {
    await button('Add Code').click();
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

    await button('Run current cell').click();

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
