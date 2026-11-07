import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import TextInput from '../TextInput';

describe('TextInput', () => {
  it('renders a labeled input and forwards changes', () => {
    const onChange = vi.fn();
    render(<TextInput label='Font size' value='14' onChange={onChange} />);
    const input = screen.getByLabelText('Font size');
    expect(input).toHaveValue('14');
    expect(input).toHaveAttribute('data-component', 'TextInput');
    fireEvent.change(input, { target: { value: '18' } });
    expect(onChange).toHaveBeenCalledOnce();
  });

  it('renders an unlabeled input when no label is provided', () => {
    render(<TextInput aria-label='Search' placeholder='Search' />);
    expect(screen.getByLabelText('Search')).toBeInTheDocument();
    expect(screen.queryByText('Search')).not.toBeInTheDocument();
  });

  it('applies input and wrapper class names', () => {
    render(
      <TextInput label='Width' wrapperClassName='wrapper' className='wide' aria-label='Width' />,
    );
    const input = screen.getByLabelText('Width');
    expect(input.className).toContain('wide');
    expect(input.closest('label')?.className).toContain('wrapper');
  });

  it('uses the aria-label for the accessible name when provided', () => {
    render(<TextInput label='Limit (ms)' aria-label='Execution timeout in milliseconds' />);
    expect(screen.getByLabelText('Execution timeout in milliseconds')).toBeInTheDocument();
  });
});
