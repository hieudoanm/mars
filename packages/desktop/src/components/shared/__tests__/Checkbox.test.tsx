import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import Checkbox from '../Checkbox';

describe('Checkbox', () => {
  it('renders a labeled checkbox', () => {
    render(<Checkbox label='Autocomplete' checked onChange={() => undefined} />);
    const checkbox = screen.getByLabelText('Autocomplete');
    expect(checkbox).toBeChecked();
    expect(checkbox).toHaveAttribute('data-component', 'Checkbox');
    expect(checkbox).toHaveAttribute('type', 'checkbox');
  });

  it('forwards click events', () => {
    const onChange = vi.fn();
    render(<Checkbox label='Folding' onChange={onChange} />);
    fireEvent.click(screen.getByLabelText('Folding'));
    expect(onChange).toHaveBeenCalledOnce();
  });

  it('applies input and wrapper class names', () => {
    render(<Checkbox label='Option' className='custom' wrapperClassName='wrapper' />);
    const checkbox = screen.getByLabelText('Option');
    expect(checkbox.className).toContain('custom');
    expect(checkbox.closest('label')?.className).toContain('wrapper');
  });
});
