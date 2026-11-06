import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import { LuSave } from 'react-icons/lu';
import IconButton from '../IconButton';

describe('IconButton', () => {
  it('renders an icon-only button with an accessible label', () => {
    render(
      <IconButton
        label='Save'
        icon={<LuSave data-testid='icon' aria-hidden='true' />}
        onClick={vi.fn()}
      />,
    );
    const button = screen.getByRole('button', { name: 'Save' });

    expect(button).toHaveAttribute('data-component', 'Button');
    expect(button).toHaveAttribute('aria-label', 'Save');
    expect(button.textContent).toBe('');
    expect(screen.getByTestId('icon')).toBeInTheDocument();
  });

  it('renders a hover and focus tooltip', () => {
    render(<IconButton label='Save' icon={<LuSave aria-hidden='true' />} />);
    const tooltip = screen.getByRole('tooltip');

    expect(tooltip).toHaveTextContent('Save');
    expect(tooltip.className).toContain('opacity-0');
    expect(tooltip.className).toContain('group-hover:opacity-100');
    expect(tooltip.className).toContain('group-focus-within:opacity-100');
  });

  it('associates the tooltip with the button', () => {
    render(<IconButton label='Save' icon={<LuSave aria-hidden='true' />} />);

    expect(screen.getByRole('button', { name: 'Save' })).toHaveAttribute(
      'aria-describedby',
      screen.getByRole('tooltip').id,
    );
  });

  it('forwards clicks, variants, and icon sizing', () => {
    const onClick = vi.fn();
    render(
      <IconButton
        label='Delete'
        variant='danger'
        icon={<LuSave aria-hidden='true' />}
        onClick={onClick}
      />,
    );
    const button = screen.getByRole('button', { name: 'Delete' });

    expect(button.className).toContain('bg-red-950');
    expect(button.className).toContain('p-2');
    fireEvent.click(button);
    expect(onClick).toHaveBeenCalledOnce();
  });
});
