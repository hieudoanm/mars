import { render, screen } from '@testing-library/react';
import Tools from '../Tools';

describe('Tools', () => {
  it('renders section heading', () => {
    render(<Tools />);
    expect(screen.getByText(/meet project mars where you work/i)).toBeInTheDocument();
    expect(screen.getByText(/one core/i)).toBeInTheDocument();
    expect(screen.getByText(/your choice of surface/i)).toBeInTheDocument();
  });

  it('renders all tool cards', () => {
    render(<Tools />);
    expect(screen.getByText(/desktop app/i)).toBeInTheDocument();
    expect(screen.getByText(/command line/i)).toBeInTheDocument();
    expect(screen.getByText(/vs code/i)).toBeInTheDocument();
  });

  it('renders tool tags', () => {
    render(<Tools />);
    expect(screen.getByText(/canonical/i)).toBeInTheDocument();
    expect(screen.getByText(/headless/i)).toBeInTheDocument();
    expect(screen.getByText(/companion/i)).toBeInTheDocument();
  });

  it('renders tool links', () => {
    render(<Tools />);
    expect(screen.getByText(/explore desktop/i)).toBeInTheDocument();
    expect(screen.getByText(/read the cli guide/i)).toBeInTheDocument();
    expect(screen.getByText(/see the extension/i)).toBeInTheDocument();
  });
});
