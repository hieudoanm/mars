import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import CellOutput from '../CellOutput';
import type { Output } from '../../../types/types';

const outputs: Output[] = [
  { type: 'text', text: 'hello stdout' },
  { type: 'error', name: 'Panic', message: 'boom', traceback: ['at foo'] },
  { type: 'html', html: '<b>bold</b>' },
  { type: 'image', mime_type: 'image/png', data: 'aGVsbG8=' },
  { type: 'table', columns: ['a', 'b'], rows: [['1', '2']] },
  { type: 'latex', latex: 'x^2' },
  { type: 'chart', spec: { kind: 'bar' } },
  { type: 'mime_bundle', data: { 'text/plain': 'plain text' } },
];

describe('CellOutput', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it('renders a labeled, collapsible output for every variant', () => {
    for (const output of outputs) {
      const { unmount } = render(<CellOutput output={output} />);
      const toggle = screen.getByRole('button', { name: /^(Collapse|Expand) / });
      expect(toggle).toBeInTheDocument();
      fireEvent.click(toggle);
      expect(toggle).toHaveAttribute('aria-expanded', 'false');
      unmount();
    }
  });

  it('copies output text to the clipboard', async () => {
    const writeText = vi.fn(async () => {});
    Object.defineProperty(navigator, 'clipboard', {
      value: { writeText },
      configurable: true,
    });
    render(<CellOutput output={{ type: 'text', text: 'copy me' }} />);
    fireEvent.click(screen.getByRole('button', { name: 'Copy output' }));
    await waitFor(() => expect(writeText).toHaveBeenCalledWith('copy me'));
  });

  it('downloads text outputs as blobs', () => {
    const createObjectURL = vi.fn(() => 'blob:mock');
    const revokeObjectURL = vi.fn();
    Object.defineProperty(URL, 'createObjectURL', { value: createObjectURL, configurable: true });
    Object.defineProperty(URL, 'revokeObjectURL', { value: revokeObjectURL, configurable: true });
    const click = vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => {});

    render(<CellOutput output={{ type: 'text', text: 'download me' }} />);
    fireEvent.click(screen.getByRole('button', { name: 'Download output' }));
    expect(createObjectURL).toHaveBeenCalled();
    expect(click).toHaveBeenCalled();
    expect(revokeObjectURL).toHaveBeenCalledWith('blob:mock');
  });

  it('downloads image outputs from base64 data', () => {
    const createObjectURL = vi.fn(() => 'blob:img');
    Object.defineProperty(URL, 'createObjectURL', { value: createObjectURL, configurable: true });
    Object.defineProperty(URL, 'revokeObjectURL', { value: vi.fn(), configurable: true });
    vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => {});

    render(<CellOutput output={{ type: 'image', mime_type: 'image/png', data: 'aGVsbG8=' }} />);
    fireEvent.click(screen.getByRole('button', { name: 'Download output' }));
    expect(createObjectURL).toHaveBeenCalled();
  });

  it('downloads mime-bundle image entries preferentially', () => {
    const createObjectURL = vi.fn(() => 'blob:mb');
    Object.defineProperty(URL, 'createObjectURL', { value: createObjectURL, configurable: true });
    Object.defineProperty(URL, 'revokeObjectURL', { value: vi.fn(), configurable: true });
    vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => {});

    render(
      <CellOutput
        output={{
          type: 'mime_bundle',
          data: { 'text/plain': 'fallback', 'image/png': 'aGVsbG8=' },
        }}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Download output' }));
    expect(createObjectURL).toHaveBeenCalled();
  });

  it('calls onRemove when provided', () => {
    const onRemove = vi.fn();
    render(<CellOutput output={{ type: 'text', text: 'x' }} onRemove={onRemove} />);
    fireEvent.click(screen.getByRole('button', { name: 'Clear output' }));
    expect(onRemove).toHaveBeenCalledTimes(1);
  });

  it('copies structured output variants as text', async () => {
    const writeText = vi.fn(async () => {});
    Object.defineProperty(navigator, 'clipboard', {
      value: { writeText },
      configurable: true,
    });
    const cases: [Output, string][] = [
      [{ type: 'error', name: 'E', message: 'm', traceback: ['t1'] }, 'E: m\nt1'],
      [{ type: 'html', html: '<i>x</i>' }, '<i>x</i>'],
      [{ type: 'image', mime_type: 'image/png', data: 'x' }, 'image/png'],
      [{ type: 'table', columns: ['a'], rows: [['1'], ['2']] }, 'a\n1\n2'],
      [{ type: 'latex', latex: 'y' }, 'y'],
      [{ type: 'chart', spec: { kind: 'line' } }, JSON.stringify({ kind: 'line' })],
      [
        { type: 'mime_bundle', data: { 'application/json': { a: 1 } } },
        JSON.stringify({ 'application/json': { a: 1 } }),
      ],
    ];
    for (const [output, expected] of cases) {
      writeText.mockClear();
      const { unmount } = render(<CellOutput output={output} />);
      fireEvent.click(screen.getByRole('button', { name: 'Copy output' }));
      await waitFor(() => expect(writeText).toHaveBeenCalledWith(expected));
      unmount();
    }
  });

  it('swallows clipboard failures', async () => {
    Object.defineProperty(navigator, 'clipboard', {
      value: { writeText: vi.fn(async () => Promise.reject(new Error('denied'))) },
      configurable: true,
    });
    render(<CellOutput output={{ type: 'text', text: 'x' }} />);
    fireEvent.click(screen.getByRole('button', { name: 'Copy output' }));
    await waitFor(() => expect(navigator.clipboard.writeText).toHaveBeenCalled());
  });

  it('downloads mime-bundle text/html entries and falls back for bad base64', () => {
    const createObjectURL = vi.fn(() => 'blob:x');
    Object.defineProperty(URL, 'createObjectURL', { value: createObjectURL, configurable: true });
    Object.defineProperty(URL, 'revokeObjectURL', { value: vi.fn(), configurable: true });
    vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => {});

    const { unmount } = render(
      <CellOutput
        output={{
          type: 'mime_bundle',
          data: { 'text/plain': 'plain', 'text/html': '<b>hi</b>' },
        }}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Download output' }));
    expect(createObjectURL).toHaveBeenCalled();
    unmount();

    // invalid base64 → base64ToBlob returns null → text fallback blob
    render(<CellOutput output={{ type: 'image', mime_type: 'image/png', data: '!!!' }} />);
    fireEvent.click(screen.getByRole('button', { name: 'Download output' }));
    expect(createObjectURL).toHaveBeenCalledTimes(2);
  });

  it('downloads remaining text-like output types', () => {
    const createObjectURL = vi.fn(() => 'blob:y');
    Object.defineProperty(URL, 'createObjectURL', { value: createObjectURL, configurable: true });
    Object.defineProperty(URL, 'revokeObjectURL', { value: vi.fn(), configurable: true });
    vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => {});

    for (const output of [
      { type: 'error', name: 'E', message: 'm', traceback: [] },
      { type: 'html', html: '<p>x</p>' },
      { type: 'table', columns: ['a'], rows: [] },
      { type: 'latex', latex: 'z' },
      { type: 'chart', spec: {} },
      { type: 'mime_bundle', data: {} },
    ] as Output[]) {
      const { unmount } = render(<CellOutput output={output} />);
      fireEvent.click(screen.getByRole('button', { name: 'Download output' }));
      unmount();
    }
    expect(createObjectURL).toHaveBeenCalled();
  });
});
