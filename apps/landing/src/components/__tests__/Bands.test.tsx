import { render, screen } from '@testing-library/react';
import Bands from '../Bands';

describe('Bands', () => {
  it('renders the band text', () => {
    render(<Bands />);
    expect(screen.getByText(/made for people who think in systems/i)).toBeInTheDocument();
  });

  it('renders category labels', () => {
    render(<Bands />);
    expect(screen.getByText(/science/i)).toBeInTheDocument();
    expect(screen.getByText(/data/i)).toBeInTheDocument();
    expect(screen.getByText(/learning/i)).toBeInTheDocument();
    expect(screen.getByText(/experiments/i)).toBeInTheDocument();
  });
});
