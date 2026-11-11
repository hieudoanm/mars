import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { TagsEditor } from '../TagsEditor';

describe('TagsEditor', () => {
  it('enters edit mode and commits parsed tags on blur', () => {
    const onUpdate = vi.fn();
    render(<TagsEditor tags={['rust', 'ml']} onUpdate={onUpdate} />);
    fireEvent.click(screen.getByLabelText('Edit cell tags'));
    const input = screen.getByLabelText('Cell tags');
    expect(input).toHaveValue('rust, ml');
    fireEvent.change(input, { target: { value: ' rust , , data ' } });
    fireEvent.blur(input);
    expect(onUpdate).toHaveBeenCalledWith(['rust', 'data']);
  });

  it('commits on Enter and cancels on Escape', () => {
    const onUpdate = vi.fn();
    render(<TagsEditor tags={[]} onUpdate={onUpdate} />);
    fireEvent.click(screen.getByLabelText('Edit cell tags'));
    const input = screen.getByLabelText('Cell tags');
    fireEvent.change(input, { target: { value: 'one' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    // Enter blurs → commit
    fireEvent.blur(input);
    expect(onUpdate).toHaveBeenCalledWith(['one']);

    fireEvent.click(screen.getByLabelText('Edit cell tags'));
    const input2 = screen.getByLabelText('Cell tags');
    fireEvent.keyDown(input2, { key: 'Escape' });
    expect(screen.queryByLabelText('Cell tags')).toBeNull();
    expect(onUpdate).toHaveBeenCalledTimes(1);
  });
});
