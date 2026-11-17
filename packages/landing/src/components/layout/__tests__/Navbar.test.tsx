import { fireEvent, render, screen } from '@testing-library/react';
import Navbar from '../Navbar';

describe('Navbar', () => {
  it('renders Project Mars home link', () => {
    render(<Navbar />);
    expect(screen.getByRole('link', { name: /project mars home/i })).toBeInTheDocument();
    expect(screen.getByText('Project Mars')).toBeInTheDocument();
  });

  it('renders primary navigation links', () => {
    render(<Navbar />);
    expect(screen.getByRole('link', { name: /why project mars/i })).toBeInTheDocument();
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

  it('toggles the mobile drawer via the hamburger button', () => {
    render(<Navbar />);
    const toggle = screen.getByRole('button', { name: /toggle navigation menu/i });
    const drawer = () => document.getElementById('mobile-navigation');
    expect(toggle).toHaveAttribute('aria-expanded', 'false');
    expect(drawer()).toHaveAttribute('aria-hidden', 'true');

    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute('aria-expanded', 'true');
    expect(drawer()).toHaveAttribute('aria-hidden', 'false');

    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute('aria-expanded', 'false');
    expect(drawer()).toHaveAttribute('aria-hidden', 'true');
  });

  it('closes the mobile drawer on Escape', () => {
    render(<Navbar />);
    fireEvent.click(screen.getByRole('button', { name: /toggle navigation menu/i }));
    expect(document.getElementById('mobile-navigation')).toHaveAttribute('aria-hidden', 'false');

    fireEvent.keyDown(window, { key: 'Escape' });
    expect(document.getElementById('mobile-navigation')).toHaveAttribute('aria-hidden', 'true');
  });
});
