import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import MobileDrawer from '../MobileDrawer';

describe('MobileDrawer', () => {
  it('is hidden when closed', () => {
    render(
      <MobileDrawer open={false} onClose={() => {}}>
        x
      </MobileDrawer>,
    );
    expect(screen.getByLabelText('Mobile navigation')).toHaveAttribute('aria-hidden', 'true');
  });

  it('renders children when open', () => {
    render(
      <MobileDrawer open={true} onClose={() => {}}>
        hello
      </MobileDrawer>,
    );
    expect(screen.getByText('hello')).toBeInTheDocument();
  });
});
