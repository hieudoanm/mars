import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import { ErrorBoundary } from '../ErrorBoundary';

const Broken = () => {
  throw new Error('boom');
};

describe('ErrorBoundary', () => {
  it('renders fallback when a child throws', () => {
    render(
      <ErrorBoundary>
        <Broken />
      </ErrorBoundary>,
    );
    expect(screen.getByText('This output could not be rendered.')).toBeInTheDocument();
  });

  it('renders children when nothing throws', () => {
    render(
      <ErrorBoundary>
        <p>fine</p>
      </ErrorBoundary>,
    );
    expect(screen.getByText('fine')).toBeInTheDocument();
  });
});
