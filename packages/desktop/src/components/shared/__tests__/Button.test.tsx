import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import { Button } from '../index';

describe('Button', () => {
  it('renders children and data-component', () => {
    render(<Button>Save</Button>);
    const button = screen.getByRole('button', { name: 'Save' });
    expect(button).toBeInTheDocument();
    expect(button).toHaveAttribute('data-component', 'Button');
  });

  it('uses button type by default', () => {
    render(<Button>Action</Button>);
    expect(screen.getByRole('button', { name: 'Action' })).toHaveAttribute('type', 'button');
  });

  it('applies variant and size classes', () => {
    render(
      <Button variant='primary' size='lg'>
        Go
      </Button>,
    );
    const button = screen.getByRole('button', { name: 'Go' });
    expect(button.className).toContain('btn-primary');
    expect(button.className).toContain('text-base');
  });

  it('applies disabled state', () => {
    const onClick = vi.fn();
    render(
      <Button disabled onClick={onClick}>
        No
      </Button>,
    );
    const button = screen.getByRole('button', { name: 'No' });
    expect(button).toBeDisabled();
    fireEvent.click(button);
    expect(onClick).not.toHaveBeenCalled();
  });

  it('applies loading state and blocks clicks', () => {
    const onClick = vi.fn();
    render(
      <Button loading onClick={onClick}>
        Submit
      </Button>,
    );
    const button = screen.getByRole('button', { name: /Submit/ });
    expect(button).toBeDisabled();
    expect(button).toHaveAttribute('aria-busy', 'true');
    expect(screen.getByText('Loading...')).toBeInTheDocument();
    fireEvent.click(button);
    expect(onClick).not.toHaveBeenCalled();
  });

  it('supports ghost and danger variants', () => {
    const { rerender } = render(<Button variant='ghost'>Ghost</Button>);
    expect(screen.getByRole('button', { name: 'Ghost' }).className).toContain('btn-ghost');
    rerender(<Button variant='danger'>Danger</Button>);
    expect(screen.getByRole('button', { name: 'Danger' }).className).toContain('btn-error');
  });

  it('supports small and medium sizes', () => {
    const { rerender } = render(<Button size='sm'>Small</Button>);
    const small = screen.getByRole('button', { name: 'Small' });
    expect(small.className).toContain('rounded-md');
    expect(small.className).toContain('px-2');
    rerender(<Button>Medium</Button>);
    const medium = screen.getByRole('button', { name: 'Medium' });
    expect(medium.className).toContain('rounded-lg');
    expect(medium.className).toContain('px-3');
  });

  it('supports icon size', () => {
    render(<Button size='icon'>X</Button>);
    expect(screen.getByRole('button', { name: 'X' }).className).toContain('size-7');
  });

  it('centers content by default and supports start alignment', () => {
    const { rerender } = render(<Button>Center</Button>);
    expect(screen.getByRole('button', { name: 'Center' }).className).toContain('justify-center');
    rerender(<Button align='start'>Start</Button>);
    expect(screen.getByRole('button', { name: 'Start' }).className).toContain('justify-start');
  });
});
