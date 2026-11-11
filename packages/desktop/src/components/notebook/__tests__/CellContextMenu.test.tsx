import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import CellContextMenu from '../CellContextMenu';

describe('CellContextMenu', () => {
  it('renders items and fires onSelect', () => {
    const onSelect = vi.fn();
    const onClose = vi.fn();
    render(
      <CellContextMenu
        x={10}
        y={10}
        onClose={onClose}
        items={[
          { label: 'Run cell', onSelect },
          { label: 'Delete', onSelect: vi.fn(), danger: true },
        ]}
      />,
    );
    fireEvent.click(screen.getByRole('menuitem', { name: 'Run cell' }));
    expect(onSelect).toHaveBeenCalledOnce();
    expect(onClose).toHaveBeenCalled();
  });

  it('closes on Escape and outside pointer down', () => {
    const onClose = vi.fn();
    render(
      <div>
        <CellContextMenu
          x={0}
          y={0}
          onClose={onClose}
          items={[{ label: 'A', onSelect: vi.fn() }]}
        />
        <button type='button'>outside</button>
      </div>,
    );
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(onClose).toHaveBeenCalled();
    fireEvent.pointerDown(screen.getByText('outside'));
    expect(onClose).toHaveBeenCalledTimes(2);
  });

  it('does not close on pointer down inside the menu and renders separators', () => {
    const onClose = vi.fn();
    render(
      <CellContextMenu
        x={0}
        y={0}
        onClose={onClose}
        items={[
          { label: 'First', onSelect: vi.fn() },
          { label: 'Second', onSelect: vi.fn(), separatorBefore: true, disabled: true },
        ]}
      />,
    );
    fireEvent.pointerDown(screen.getByRole('menuitem', { name: 'First' }));
    expect(onClose).not.toHaveBeenCalled();
    expect(document.querySelector('.border-t')).not.toBeNull();
    expect(screen.getByRole('menuitem', { name: 'Second' })).toBeDisabled();
    fireEvent(window, new Event('blur'));
    expect(onClose).toHaveBeenCalled();
  });

  it('clamps the menu position inside the viewport', () => {
    render(
      <CellContextMenu
        x={10000}
        y={10000}
        onClose={vi.fn()}
        items={[{ label: 'A', onSelect: vi.fn() }]}
      />,
    );
    const menu = screen.getByRole('menu');
    expect(Number.parseInt(menu.style.left)).toBeLessThanOrEqual(window.innerWidth - 220);
    expect(Number.parseInt(menu.style.top)).toBeLessThanOrEqual(window.innerHeight - 48);
  });
});
