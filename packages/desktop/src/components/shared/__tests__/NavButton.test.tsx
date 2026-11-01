import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import NavButton from '../NavButton';

describe('NavButton', () => {
  it('renders a button with the default type', () => {
    render(<NavButton>markdown</NavButton>);
    const button = screen.getByRole('button', { name: 'markdown' });
    expect(button).toHaveAttribute('type', 'button');
    expect(button).toHaveAttribute('data-component', 'NavButton');
    expect(button).not.toHaveAttribute('aria-current');
  });

  it('highlights the active item', () => {
    render(
      <NavButton active index='01'>
        code
      </NavButton>,
    );
    const button = screen.getByRole('button', { name: '01code' });
    expect(button.className).toContain('bg-orange-950');
    expect(button).toHaveAttribute('aria-current', 'true');
  });

  it('forwards clicks', () => {
    const onClick = vi.fn();
    render(<NavButton onClick={onClick}>raw</NavButton>);
    fireEvent.click(screen.getByRole('button', { name: 'raw' }));
    expect(onClick).toHaveBeenCalledOnce();
  });
});
