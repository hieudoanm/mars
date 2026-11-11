import { fireEvent, render, screen } from '@testing-library/react';
import { beforeAll, describe, expect, it, vi } from 'vitest';
import CommandPalette, { type PaletteCommand } from '../CommandPalette';

beforeAll(() => {
  Element.prototype.scrollIntoView = vi.fn();
});

const commands: PaletteCommand[] = [
  { id: 'a', title: 'Save notebook', hint: 'mod+s', run: vi.fn() },
  { id: 'b', title: 'Run all cells', hint: 'f6', run: vi.fn() },
  { id: 'c', title: 'Toggle sidebar', run: vi.fn() },
];

const setup = (overrides: Partial<Parameters<typeof CommandPalette>[0]> = {}) => {
  const props = {
    open: true,
    commands,
    onClose: vi.fn(),
    ...overrides,
  };
  render(<CommandPalette {...props} />);
  return props;
};

describe('CommandPalette', () => {
  it('renders nothing when closed', () => {
    const { container } = render(
      <CommandPalette open={false} commands={commands} onClose={vi.fn()} />,
    );
    expect(container).toBeEmptyDOMElement();
  });

  it('filters commands by title and hint', () => {
    setup();
    const input = screen.getByPlaceholderText(/search|command/i) ?? screen.getByRole('textbox');
    fireEvent.change(input, { target: { value: 'sidebar' } });
    expect(screen.getByText('Toggle sidebar')).toBeInTheDocument();
    expect(screen.queryByText('Save notebook')).toBeNull();
    fireEvent.change(input, { target: { value: 'f6' } });
    expect(screen.getByText('Run all cells')).toBeInTheDocument();
  });

  it('runs the selected command on click and Enter', () => {
    const run = vi.fn();
    const props = setup({ commands: [{ id: 'x', title: 'Do thing', run }] });
    fireEvent.click(screen.getByText('Do thing'));
    expect(run).toHaveBeenCalledTimes(1);
    expect(props.onClose).toHaveBeenCalledTimes(1);
  });

  it('navigates with arrow keys and runs on Enter', () => {
    const runB = vi.fn();
    const cmds = [
      { id: 'a', title: 'Alpha', run: vi.fn() },
      { id: 'b', title: 'Beta', run: runB },
    ];
    setup({ commands: cmds });
    fireEvent.keyDown(window, { key: 'ArrowDown' });
    fireEvent.keyDown(window, { key: 'Enter' });
    expect(runB).toHaveBeenCalled();
  });

  it('closes on Escape', () => {
    const props = setup();
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(props.onClose).toHaveBeenCalled();
  });

  it('shows an empty state when nothing matches', () => {
    setup();
    fireEvent.change(screen.getByRole('textbox'), { target: { value: 'zzz' } });
    expect(screen.queryByText('Save notebook')).toBeNull();
  });
});
