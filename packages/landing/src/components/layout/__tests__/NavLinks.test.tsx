import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import NavLinks from '../NavLinks';

describe('NavLinks', () => {
  it('renders primary links', () => {
    render(<NavLinks />);
    expect(screen.getByRole('link', { name: /why project mars/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /downloads/i })).toBeInTheDocument();
  });

  it('renders home when showHome', () => {
    render(<NavLinks showHome />);
    expect(screen.getByRole('link', { name: /home/i })).toBeInTheDocument();
  });
});
