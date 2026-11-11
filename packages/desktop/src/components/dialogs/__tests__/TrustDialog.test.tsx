import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import TrustDialog from '../TrustDialog';

describe('TrustDialog', () => {
  it('shows the file name and forwards trust/cancel actions', () => {
    const onTrust = vi.fn();
    const onCancel = vi.fn();
    render(<TrustDialog fileName='evil.mars' onTrust={onTrust} onCancel={onCancel} />);
    expect(screen.getByText('evil.mars')).toBeInTheDocument();
    expect(screen.getByText(/contains executable Rust code/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: /trust & open/i }));
    expect(onTrust).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(onCancel).toHaveBeenCalledTimes(1);
  });
});
