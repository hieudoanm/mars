import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';

const { embed } = vi.hoisted(() => ({ embed: vi.fn() }));
vi.mock('vega-embed', () => ({ default: embed }));

import OutputView from '../OutputView';
import type { Output } from '../../../types/types';

describe('OutputView', () => {
  beforeEach(() => {
    embed.mockReset().mockResolvedValue({ view: { finalize: vi.fn() } });
  });

  it('renders text and error outputs with diagnostics', () => {
    const { rerender } = render(<OutputView output={{ type: 'text', text: 'plain text' }} />);
    expect(screen.getByText('plain text')).toBeInTheDocument();
    rerender(
      <OutputView
        output={{ type: 'error', name: 'CompileError', message: 'bad expression', traceback: [] }}
      />,
    );
    expect(screen.getByText(/CompileError/)).toBeInTheDocument();
    expect(screen.getByText(/bad expression/)).toBeInTheDocument();
  });

  it('sanitizes HTML and renders LaTeX', () => {
    const { rerender } = render(
      <OutputView output={{ type: 'html', html: '<p>safe</p><script>bad()</script>' }} />,
    );
    expect(screen.getByText('safe')).toBeInTheDocument();
    expect(document.querySelector('script')).toBeNull();
    rerender(<OutputView output={{ type: 'latex', latex: 'x^2' }} />);
    expect(document.querySelector('[data-latex]')).toHaveAttribute('data-latex', 'x^2');
  });

  it('renders images and tables', () => {
    const { rerender } = render(
      <OutputView output={{ type: 'image', mime_type: 'image/png', data: 'aGVsbG8=' }} />,
    );
    expect(screen.getByRole('img', { name: 'Notebook output' })).toHaveAttribute(
      'src',
      'data:image/png;base64,aGVsbG8=',
    );
    rerender(
      <OutputView output={{ type: 'table', columns: ['Name', 'Value'], rows: [['Project Mars', '42']] }} />,
    );
    expect(screen.getByRole('columnheader', { name: 'Name' })).toBeInTheDocument();
    expect(screen.getByRole('cell', { name: '42' })).toBeInTheDocument();
  });

  it('renders chart specs and safely falls back for invalid chart JSON', () => {
    const spec = { mark: 'bar', data: { values: [] } };
    const { rerender } = render(<OutputView output={{ type: 'chart', spec }} />);
    rerender(<OutputView output={{ type: 'chart', spec: { title: 'Project Mars' } }} />);
    expect(embed).toHaveBeenCalled();
    rerender(
      <OutputView
        output={{ type: 'mime_bundle', data: { 'application/vnd.vega.v5+json': '{invalid' } }}
      />,
    );
    expect(screen.getByText('{invalid')).toBeInTheDocument();
  });

  it('shows chart renderer failures to the user', async () => {
    embed.mockRejectedValueOnce(new Error('Invalid Vega specification'));
    render(<OutputView output={{ type: 'chart', spec: { mark: 'unknown' } }} />);
    expect(
      await screen.findByText(/Chart rendering failed: Invalid Vega specification/),
    ).toBeInTheDocument();
  });

  it('chooses rich MIME outputs and falls back to readable data', () => {
    const cases: Output[] = [
      { type: 'mime_bundle', data: {} },
      { type: 'mime_bundle', data: { 'text/plain': ['hello', ' world'] } },
      { type: 'mime_bundle', data: { 'text/markdown': '# Markdown' } },
      { type: 'mime_bundle', data: { 'text/latex': 'y=mx' } },
      { type: 'mime_bundle', data: { 'text/html': '<b>rich</b>' } },
      { type: 'mime_bundle', data: { 'image/svg+xml': '<svg></svg>' } },
    ];
    for (const output of cases) {
      const { unmount } = render(<OutputView output={output} />);
      unmount();
    }
    expect(embed).not.toHaveBeenCalled();
  });

  it('renders runtime source annotations as navigation buttons', () => {
    const onNavigateSource = vi.fn();
    render(
      <OutputView
        output={{
          type: 'error',
          name: 'RuntimeError',
          message: "thread 'main' panicked at src/main.rs:5:5: [mars:cell=42,line=2]",
          traceback: [],
        }}
        onNavigateSource={onNavigateSource}
      />,
    );
    const button = screen.getByRole('button', { name: '[mars:cell=42,line=2]' });
    fireEvent.click(button);
    expect(onNavigateSource).toHaveBeenCalledWith(42, 2);
  });
});
