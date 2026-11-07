import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import HamburgerButton from '../HamburgerButton';

describe('HamburgerButton', () => {
  it('renders menu icon when closed', () => {
    render(<HamburgerButton open={false} onClick={() => {}} />);
    expect(screen.getByRole('button', { name: /toggle navigation menu/i })).toBeInTheDocument();
  });

  it('calls onClick', () => {
    const onClick = vi.fn();
    render(<HamburgerButton open={false} onClick={onClick} />);
    fireEvent.click(screen.getByRole('button'));
    expect(onClick).toHaveBeenCalled();
  });
});
