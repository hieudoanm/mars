import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import Badge from '../Badge';

describe('Badge', () => {
  it('renders neutral text without a dot by default', () => {
    const { container } = render(<Badge>Saved</Badge>);
    const badge = screen.getByText('Saved');
    expect(badge).toHaveAttribute('data-component', 'Badge');
    expect(badge.className).toContain('text-neutral-400');
    expect(container.querySelector('[aria-hidden="true"]')).toBeNull();
  });

  it('renders tone-specific text and dot colors', () => {
    render(
      <Badge tone='warning' dot dotTone='accent' dotClassName='custom-dot'>
        Unsaved
      </Badge>,
    );
    const badge = screen.getByText('Unsaved');
    expect(badge.className).toContain('text-orange-300');
    const dot = badge.querySelector('[aria-hidden="true"]');
    expect(dot?.className).toContain('bg-orange-500');
    expect(dot?.className).toContain('custom-dot');
  });

  it('merges custom class names', () => {
    render(<Badge className='custom'>Status</Badge>);
    expect(screen.getByText('Status').className).toContain('custom');
  });
});
