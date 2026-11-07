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
});
