import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import Kbd from '../Kbd';

describe('Kbd', () => {
  it('renders a keyboard hint', () => {
    render(<Kbd>F6</Kbd>);
    const hint = screen.getByText('F6');
    expect(hint.tagName).toBe('KBD');
    expect(hint).toHaveAttribute('data-component', 'Kbd');
  });

  it('merges custom class names', () => {
    render(<Kbd className='custom'>Ctrl</Kbd>);
    expect(screen.getByText('Ctrl').className).toContain('custom');
  });
});
