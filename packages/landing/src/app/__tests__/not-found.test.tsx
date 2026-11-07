import { render, screen } from '@testing-library/react';
import NotFound from '../not-found';

describe('NotFound', () => {
  it('renders page not found heading', () => {
    render(<NotFound />);
    expect(screen.getByText(/lost in space/i)).toBeInTheDocument();
  });

  it('renders back to docs link', () => {
    render(<NotFound />);
    expect(screen.getByRole('link', { name: /back to docs/i })).toHaveAttribute(
      'href',
      '/docs',
    );
  });

  it('renders Mars logo', () => {
    render(<NotFound />);
    expect(screen.getByAltText('')).toBeInTheDocument();
  });
});
