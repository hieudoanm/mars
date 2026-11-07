import { render, screen } from '@testing-library/react';
import Navbar from '../Navbar';

describe('Navbar', () => {
  it('renders Mars home link', () => {
    render(<Navbar />);
    expect(screen.getByRole('link', { name: /mars home/i })).toBeInTheDocument();
    expect(screen.getByText('Mars')).toBeInTheDocument();
  });

  it('renders primary navigation links', () => {
    render(<Navbar />);
    expect(screen.getByRole('link', { name: /why mars/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /workflow/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /architecture/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /downloads/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /get started/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /docs/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /blog/i })).toBeInTheDocument();
  });

  it('renders GitHub link', () => {
    render(<Navbar />);
    expect(screen.getByRole('link', { name: /github/i })).toBeInTheDocument();
  });

  it('renders header with primary navigation aria-label', () => {
    render(<Navbar />);
    expect(screen.getByRole('navigation', { name: 'Primary navigation' })).toBeInTheDocument();
  });
});
