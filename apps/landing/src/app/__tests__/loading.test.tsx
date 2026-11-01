import { render, screen } from '@testing-library/react';
import Loading from '../loading';

describe('Loading', () => {
  it('renders Mars logo and loading message', () => {
    render(<Loading />);
    expect(screen.getByAltText('')).toBeInTheDocument();
    expect(screen.getByText(/loading mars/i)).toBeInTheDocument();
  });
});
