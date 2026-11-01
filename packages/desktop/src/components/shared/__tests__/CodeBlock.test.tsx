import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import CodeBlock from '../CodeBlock';

describe('CodeBlock', () => {
  it('renders preformatted content', () => {
    render(<CodeBlock>hello</CodeBlock>);
    const block = screen.getByText('hello');
    expect(block.tagName).toBe('PRE');
    expect(block).toHaveAttribute('data-component', 'CodeBlock');
    expect(block.className).toContain('text-neutral-300');
  });

  it('supports muted and danger tones', () => {
    const { rerender } = render(<CodeBlock tone='muted'>muted</CodeBlock>);
    expect(screen.getByText('muted').className).toContain('text-neutral-400');

    rerender(<CodeBlock tone='danger'>error</CodeBlock>);
    const error = screen.getByText('error');
    expect(error.className).toContain('border-red-500');
    expect(error.className).toContain('text-red-200');
  });

  it('merges custom class names', () => {
    render(<CodeBlock className='custom'>code</CodeBlock>);
    expect(screen.getByText('code').className).toContain('custom');
  });
});
