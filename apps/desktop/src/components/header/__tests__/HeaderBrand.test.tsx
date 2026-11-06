import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import HeaderBrand from '../HeaderBrand';

describe('HeaderBrand', () => {
  it('renders the Mars logo, brand label, and headline', () => {
    render(<HeaderBrand />);
    const logo = screen.getByAltText('Mars');
    expect(logo).toHaveAttribute('src', '/branding/svg/mars-logo.svg');
    expect(screen.getByText('MARS')).toBeInTheDocument();
    expect(screen.getByRole('heading', { level: 1, name: 'Explore ideas.' })).toBeInTheDocument();
  });

  it('exposes the component marker', () => {
    const { container } = render(<HeaderBrand />);
    expect(container.querySelector('[data-component="HeaderBrand"]')).toBeInTheDocument();
  });
});
