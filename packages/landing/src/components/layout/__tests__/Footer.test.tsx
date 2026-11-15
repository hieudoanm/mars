import { render, screen } from '@testing-library/react';
import Footer from '../Footer';

describe('Footer', () => {
  it('renders Project Mars home link', () => {
    render(<Footer />);
    expect(screen.getByRole('link', { name: /mars/i })).toBeInTheDocument();
  });

  it('renders tagline', () => {
    render(<Footer />);
    expect(screen.getByText(/explore ideas/i)).toBeInTheDocument();
    expect(screen.getByText(/execute code/i)).toBeInTheDocument();
    expect(screen.getByText(/discover possibilities/i)).toBeInTheDocument();
  });

  it('renders license and build info', () => {
    render(<Footer />);
    expect(screen.getByText(/mit licensed/i)).toBeInTheDocument();
    expect(screen.getByText(/built with rust/i)).toBeInTheDocument();
  });
});
