import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import SectionHeading from '../SectionHeading';

describe('SectionHeading', () => {
  it('renders a level-two heading by default', () => {
    render(<SectionHeading>Notebook</SectionHeading>);
    const heading = screen.getByRole('heading', { level: 2, name: 'Notebook' });
    expect(heading).toHaveAttribute('data-component', 'SectionHeading');
  });

  it('supports alternative semantics', () => {
    const { rerender } = render(<SectionHeading as='h3'>Cells</SectionHeading>);
    expect(screen.getByRole('heading', { level: 3, name: 'Cells' })).toBeInTheDocument();

    rerender(<SectionHeading as='div'>Cells</SectionHeading>);
    expect(screen.queryByRole('heading', { name: 'Cells' })).not.toBeInTheDocument();
    expect(screen.getByText('Cells')).toBeInTheDocument();
  });

  it('merges custom class names', () => {
    render(<SectionHeading className='custom'>Heading</SectionHeading>);
    expect(screen.getByRole('heading', { name: 'Heading' }).className).toContain('custom');
  });
});
