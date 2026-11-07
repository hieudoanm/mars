import { render, screen } from '@testing-library/react';
import Workflow from '../Workflow';

describe('Workflow', () => {
  it('renders section heading', () => {
    render(<Workflow />);
    expect(screen.getByText(/one document. many ways forward/i)).toBeInTheDocument();
    expect(screen.getByText(/a workflow that follows/i)).toBeInTheDocument();
    expect(screen.getByText(/your curiosity/i)).toBeInTheDocument();
  });

  it('renders all workflow steps', () => {
    render(<Workflow />);
    expect(screen.getByText(/write/i)).toBeInTheDocument();
    expect(screen.getByText(/run/i)).toBeInTheDocument();
    expect(screen.getByText(/see/i)).toBeInTheDocument();
    expect(screen.getByText(/share/i)).toBeInTheDocument();
  });

  it('renders step numbers', () => {
    render(<Workflow />);
    expect(screen.getByText(/01/)).toBeInTheDocument();
    expect(screen.getByText(/02/)).toBeInTheDocument();
    expect(screen.getByText(/03/)).toBeInTheDocument();
    expect(screen.getByText(/04/)).toBeInTheDocument();
  });

  it('renders step descriptions', () => {
    render(<Workflow />);
    expect(screen.getByText(/capture the thought in code or markdown/i)).toBeInTheDocument();
    expect(screen.getByText(/execute rust with persistent bindings/i)).toBeInTheDocument();
    expect(screen.getByText(/turn results into rich, useful output/i)).toBeInTheDocument();
    expect(screen.getByText(/export, reproduce, and keep moving/i)).toBeInTheDocument();
  });
});
