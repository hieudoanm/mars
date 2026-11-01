import { render, screen } from '@testing-library/react';
import BuiltDifferent from '../BuiltDifferent';

describe('BuiltDifferent', () => {
  it('renders section heading', () => {
    render(<BuiltDifferent />);
    expect(screen.getByText(/built differently, on purpose/i)).toBeInTheDocument();
    expect(screen.getByText(/not a web wrapper/i)).toBeInTheDocument();
    expect(screen.getByText(/not a black box/i)).toBeInTheDocument();
  });

  it('renders architecture components', () => {
    render(<BuiltDifferent />);
    expect(screen.getByText(/notebook ui/i)).toBeInTheDocument();
    expect(screen.getByText(/typed protocol/i)).toBeInTheDocument();
    expect(screen.getByText(/rust execution/i)).toBeInTheDocument();
  });

  it('renders feature list items', () => {
    render(<BuiltDifferent />);
    expect(screen.getByText(/rust-first foundation/i)).toBeInTheDocument();
    expect(screen.getByText(/reproducible execution/i)).toBeInTheDocument();
    expect(screen.getByText(/open interoperability/i)).toBeInTheDocument();
  });

  it('renders better experiments text', () => {
    render(<BuiltDifferent />);
    expect(screen.getByText(/clear boundaries/i)).toBeInTheDocument();
    expect(screen.getByText(/better experiments/i)).toBeInTheDocument();
  });
});
