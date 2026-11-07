import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import NotebookHeaderBrand from '../NotebookHeaderBrand';

describe('NotebookHeaderBrand', () => {
  it('renders the Mars logo, brand label, and headline', () => {
    render(<NotebookHeaderBrand />);
    const logo = screen.getByAltText('Mars');
    expect(logo).toHaveAttribute('src');
    expect(screen.getByRole('heading', { level: 1, name: 'Mars' })).toBeInTheDocument();
    expect(screen.getByText('Explore ideas.')).toBeInTheDocument();
  });

  it('exposes the component marker', () => {
    const { container } = render(<NotebookHeaderBrand />);
    expect(container.querySelector('[data-component="NotebookHeaderBrand"]')).toBeInTheDocument();
  });
});
