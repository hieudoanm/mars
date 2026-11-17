import { act, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import Demo from '../Demo';

describe('Demo', () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it('renders the demo window and notebook cells', () => {
    render(<Demo />);
    expect(screen.getByText('See it in action')).toBeInTheDocument();
    expect(screen.getByText('analysis.mars — Project Mars')).toBeInTheDocument();
    expect(screen.getByText(/load_signal/)).toBeInTheDocument();
    expect(screen.getByText(/visualize/)).toBeInTheDocument();
    expect(screen.getByText('FILES')).toBeInTheDocument();
    expect(screen.getByText('VARIABLES')).toBeInTheDocument();
  });

  it('animates cells through executing and completed states', async () => {
    vi.useFakeTimers();
    render(<Demo />);

    // First cell starts executing after the initial 1s pause.
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1_200);
    });
    expect(screen.getAllByText('Executing...').length).toBeGreaterThan(0);

    // By ~7s all three cells have produced their outputs (before the loop resets).
    await act(async () => {
      await vi.advanceTimersByTimeAsync(6_000);
    });
    expect(screen.getByText(/Signal loaded: 1024 samples/)).toBeInTheDocument();
    expect(screen.getByText(/Mean calculated: 42\.3/)).toBeInTheDocument();
    expect(screen.getByText(/Chart rendered in 12ms/)).toBeInTheDocument();
  });
});
