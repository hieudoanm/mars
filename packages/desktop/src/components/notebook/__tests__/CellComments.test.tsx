import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { CellComments } from '../CellComments';
import type { CellComment } from '../../../lib/cellComments';

const comment = (text: string): CellComment => ({
  text,
  created_at: '2026-01-01T00:00:00Z',
});

describe('CellComments', () => {
  it('renders an empty state', () => {
    render(<CellComments comments={[]} onChange={() => {}} />);
    expect(screen.getByText('Comments')).toBeInTheDocument();
    expect(screen.getByLabelText('Add a comment')).toBeInTheDocument();
  });

  it('shows existing comments with count', () => {
    render(<CellComments comments={[comment('first'), comment('second')]} onChange={() => {}} />);
    expect(screen.getByText('Comments (2)')).toBeInTheDocument();
    expect(screen.getByText('first')).toBeInTheDocument();
    expect(screen.getByText('second')).toBeInTheDocument();
  });

  it('adds a comment via the Add button', () => {
    const onChange = vi.fn();
    render(<CellComments comments={[]} onChange={onChange} />);
    fireEvent.change(screen.getByLabelText('Add a comment'), {
      target: { value: '  nice work  ' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Add' }));
    expect(onChange).toHaveBeenCalledWith([expect.objectContaining({ text: 'nice work' })]);
  });

  it('adds a comment on Enter', () => {
    const onChange = vi.fn();
    render(<CellComments comments={[]} onChange={onChange} />);
    const input = screen.getByLabelText('Add a comment');
    fireEvent.change(input, { target: { value: 'via enter' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(onChange).toHaveBeenCalledWith([expect.objectContaining({ text: 'via enter' })]);
  });

  it('keeps the add button disabled for blank input', () => {
    render(<CellComments comments={[]} onChange={() => {}} />);
    expect(screen.getByRole('button', { name: 'Add' })).toBeDisabled();
  });

  it('does not call onChange for blank submissions', () => {
    const onChange = vi.fn();
    render(<CellComments comments={[comment('x')]} onChange={onChange} />);
    fireEvent.keyDown(screen.getByLabelText('Add a comment'), { key: 'Enter' });
    expect(onChange).not.toHaveBeenCalled();
  });

  it('deletes a comment', () => {
    const onChange = vi.fn();
    render(<CellComments comments={[comment('keep'), comment('drop')]} onChange={onChange} />);
    const deleteButtons = screen.getAllByLabelText('Delete comment');
    fireEvent.click(deleteButtons[1]);
    expect(onChange).toHaveBeenCalledWith([expect.objectContaining({ text: 'keep' })]);
  });
});
