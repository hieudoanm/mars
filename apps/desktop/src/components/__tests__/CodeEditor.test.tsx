import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import CodeEditor from '../CodeEditor';

describe('CodeEditor', () => {
  it('creates a Rust editor with a labeled input and applies document updates', () => {
    const onChange = vi.fn();
    const onFocus = vi.fn();
    const { rerender, unmount } = render(
      <CodeEditor
        cellType='code'
        value='let answer = 42;'
        placeholder='Write Rust code...'
        onChange={onChange}
        onFocus={onFocus}
      />,
    );
    expect(screen.getByRole('textbox', { name: 'Write Rust code...' })).toBeInTheDocument();
    expect(document.querySelector('.cm-content')?.textContent).toContain('let answer = 42;');
    rerender(
      <CodeEditor
        cellType='markdown'
        value='# Updated'
        placeholder='Write Markdown...'
        onChange={onChange}
        onFocus={onFocus}
        diagnostics={[]}
        autocompleteEnabled={false}
        foldingEnabled={false}
        fontSize={18}
      />,
    );
    expect(screen.getByRole('textbox', { name: 'Write Markdown...' })).toBeInTheDocument();
    expect(document.querySelector('.cm-content')?.textContent).toContain('# Updated');
    unmount();
    expect(document.querySelector('.cm-editor')).toBeNull();
  });
});
