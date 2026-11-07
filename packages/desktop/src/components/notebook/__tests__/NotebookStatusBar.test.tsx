import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import NotebookStatusBar from '../NotebookStatusBar';

describe('NotebookStatusBar', () => {
  it('renders cell count and saved state', () => {
    render(<NotebookStatusBar cellCount={2} isDirty={false} />);
    expect(screen.getByText('Rust 2024')).toBeInTheDocument();
    expect(screen.getByText('Kernel idle')).toBeInTheDocument();
    expect(screen.getByText('2 cells')).toBeInTheDocument();
    expect(screen.getByText('All changes saved')).toBeInTheDocument();
  });

  it('renders unsaved changes state', () => {
    render(<NotebookStatusBar cellCount={5} isDirty />);
    expect(screen.getByText('5 cells')).toBeInTheDocument();
    expect(screen.getByText('Unsaved changes')).toBeInTheDocument();
  });

  it('shows the current edit mode', () => {
    const { rerender } = render(<NotebookStatusBar cellCount={1} isDirty={false} mode='edit' />);
    expect(screen.getByText('Edit')).toBeInTheDocument();
    rerender(<NotebookStatusBar cellCount={1} isDirty={false} mode='command' />);
    expect(screen.getByText('Command')).toBeInTheDocument();
  });

  it('renders zero cells', () => {
    render(<NotebookStatusBar cellCount={0} isDirty={false} />);
    expect(screen.getByText('0 cells')).toBeInTheDocument();
  });
});
