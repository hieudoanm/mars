import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import PromptDialog from '../PromptDialog';

describe('PromptDialog', () => {
  it('submits the typed answer and resets', () => {
    const onSubmit = vi.fn();
    render(<PromptDialog message='Name?' onSubmit={onSubmit} />);
    expect(screen.getByText('Name?')).toBeInTheDocument();
    const input = screen.getByLabelText('Prompt answer');
    fireEvent.change(input, { target: { value: 'mars' } });
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    expect(onSubmit).toHaveBeenCalledWith('mars');
    expect(input).toHaveValue('');
  });

  it('submits via form submit', () => {
    const onSubmit = vi.fn();
    render(<PromptDialog message='q' onSubmit={onSubmit} />);
    const input = screen.getByLabelText('Prompt answer');
    fireEvent.change(input, { target: { value: '42' } });
    fireEvent.submit(input.closest('form')!);
    expect(onSubmit).toHaveBeenCalledWith('42');
  });
});
