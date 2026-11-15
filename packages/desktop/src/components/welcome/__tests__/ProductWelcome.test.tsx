import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import ProductWelcome from '../ProductWelcome';

describe('ProductWelcome', () => {
  it('renders both product cards', () => {
    render(<ProductWelcome onSelect={() => {}} />);
    expect(screen.getByText('Welcome to Project Mars')).toBeInTheDocument();
    expect(screen.getByText('Mars Lab')).toBeInTheDocument();
    expect(screen.getByText('Mars Notebook')).toBeInTheDocument();
  });

  it('selects lab when the lab card is clicked', () => {
    const onSelect = vi.fn();
    render(<ProductWelcome onSelect={onSelect} />);
    fireEvent.click(screen.getByRole('button', { name: /Mars Lab/ }));
    expect(onSelect).toHaveBeenCalledWith('lab');
  });

  it('selects notebook when the notebook card is clicked', () => {
    const onSelect = vi.fn();
    render(<ProductWelcome onSelect={onSelect} />);
    fireEvent.click(screen.getByRole('button', { name: /Mars Notebook/ }));
    expect(onSelect).toHaveBeenCalledWith('notebook');
  });
});
