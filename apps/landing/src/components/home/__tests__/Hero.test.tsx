import { render, screen } from '@testing-library/react';
import Hero from '../Hero';

describe('Hero', () => {
  it('renders hero heading', () => {
    render(<Hero />);
    expect(screen.getByText(/think in cells/i)).toBeInTheDocument();
    expect(screen.getByText(/build in rust/i)).toBeInTheDocument();
  });

  it('renders eyebrow and description', () => {
    render(<Hero />);
    expect(screen.getByText(/rust-powered interactive notebooks/i)).toBeInTheDocument();
    expect(screen.getByText(/focused environment/i)).toBeInTheDocument();
  });

  it('renders primary and quiet buttons', () => {
    render(<Hero />);
    expect(screen.getByRole('link', { name: /start exploring/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /view on github/i })).toBeInTheDocument();
  });

  it('renders feature highlights', () => {
    render(<Hero />);
    expect(screen.getByText(/native/i)).toBeInTheDocument();
    expect(screen.getByText(/rust execution/i)).toBeInTheDocument();
    expect(screen.getByText(/jupyter import\/export/i)).toBeInTheDocument();
  });

  it('renders notebook interface preview with aria-label', () => {
    render(<Hero />);
    expect(screen.getByRole('img', { name: /mars logo/i })).toBeInTheDocument();
  });
});
