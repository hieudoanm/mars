import { render, screen } from '@testing-library/react';
import App from './LandingPage';

describe('LandingPage', () => {
  it('renders Mars title', () => {
    render(<App />);
    const marsElements = screen.getAllByText('Mars');
    expect(marsElements.length).toBeGreaterThan(0);
  });

  it('renders main heading', () => {
    render(<App />);
    expect(screen.getByText(/Think in cells/i)).toBeInTheDocument();
  });
});
