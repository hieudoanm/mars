import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import DependenciesDialog from '../DependenciesDialog';

const setup = (deps: Record<string, string> = { serde: '1.0' }) => {
  const onSave = vi.fn();
  const onClose = vi.fn();
  render(<DependenciesDialog open dependencies={deps} onSave={onSave} onClose={onClose} />);
  return { onSave, onClose };
};

describe('DependenciesDialog', () => {
  it('renders nothing when closed', () => {
    const { container } = render(
      <DependenciesDialog open={false} dependencies={{}} onSave={vi.fn()} onClose={vi.fn()} />,
    );
    expect(container).toBeEmptyDOMElement();
  });

  it('lists existing dependencies and edits versions', () => {
    const { onSave } = setup({ serde: '1.0', tokio: '1.40' });
    expect(screen.getByText('serde')).toBeInTheDocument();
    expect(screen.getByText('tokio')).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText('Version for serde'), { target: { value: '1.1' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save dependencies' }));
    expect(onSave).toHaveBeenCalledWith({ serde: '1.1', tokio: '1.40' });
  });

  it('adds a dependency with a default wildcard version', () => {
    const { onSave } = setup({});
    fireEvent.change(screen.getByLabelText('Dependency name'), { target: { value: 'anyhow' } });
    fireEvent.click(screen.getByRole('button', { name: 'Add dependency' }));
    expect(screen.getByText('anyhow')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Save dependencies' }));
    expect(onSave).toHaveBeenCalledWith({ anyhow: '*' });
  });

  it('adds via Enter key and ignores empty names', () => {
    setup({});
    const name = screen.getByLabelText('Dependency name');
    fireEvent.keyDown(name, { key: 'Enter' });
    expect(screen.queryByText('No dependencies.')).not.toBeNull();
    fireEvent.change(name, { target: { value: 'clap' } });
    fireEvent.change(screen.getByLabelText('Dependency version'), { target: { value: '4' } });
    fireEvent.keyDown(screen.getByLabelText('Dependency version'), { key: 'Enter' });
    expect(screen.getByText('clap')).toBeInTheDocument();
  });

  it('replaces an existing dependency with the same name', () => {
    const { onSave } = setup({ serde: '1.0' });
    fireEvent.change(screen.getByLabelText('Dependency name'), { target: { value: 'serde' } });
    fireEvent.change(screen.getByLabelText('Dependency version'), { target: { value: '2.0' } });
    fireEvent.click(screen.getByRole('button', { name: 'Add dependency' }));
    fireEvent.click(screen.getByRole('button', { name: 'Save dependencies' }));
    expect(onSave).toHaveBeenCalledWith({ serde: '2.0' });
  });

  it('removes a dependency row', () => {
    const { onSave } = setup({ serde: '1.0', tokio: '1' });
    fireEvent.click(screen.getByRole('button', { name: 'Remove serde' }));
    expect(screen.queryByText('serde')).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Save dependencies' }));
    expect(onSave).toHaveBeenCalledWith({ tokio: '1' });
  });

  it('cancels and closes', () => {
    const { onClose } = setup();
    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(onClose).toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Close' }));
    expect(onClose).toHaveBeenCalledTimes(2);
  });
});
