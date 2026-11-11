import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { NotebookVariables } from '../NotebookVariables';
import type { KernelVariable } from '../../../types/types';
import type { RustSymbol } from '../../../lib/rustSymbols';

const variables: KernelVariable[] = [
  { name: 'count', typeName: 'i32', value: '42' },
  { name: 'name', typeName: 'String', value: null },
];

const symbols: RustSymbol[] = [
  { name: 'main', kind: 'function' },
  { name: 'Config', kind: 'type' },
];

describe('NotebookVariables', () => {
  it('renders nothing when empty', () => {
    const { container } = render(<NotebookVariables liveVariables={[]} symbols={[]} />);
    expect(container).toBeEmptyDOMElement();
  });

  it('lists live variables with types and copy buttons', () => {
    const onRefresh = vi.fn();
    Object.defineProperty(navigator, 'clipboard', {
      value: { writeText: vi.fn(async () => {}) },
      configurable: true,
    });
    render(
      <NotebookVariables liveVariables={variables} symbols={[]} onRefreshVariables={onRefresh} />,
    );
    expect(screen.getByText('count')).toBeInTheDocument();
    expect(screen.getByText('i32')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Refresh variables' }));
    expect(onRefresh).toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Copy count' }));
  });

  it('expands and collapses variable values', () => {
    render(<NotebookVariables liveVariables={variables} symbols={[]} />);
    const value = screen.getByTitle('Expand value');
    expect(value.querySelector('span')).toHaveClass('truncate');
    fireEvent.click(value);
    expect(screen.getByTitle('Collapse value').querySelector('span')).toHaveClass(
      'whitespace-pre-wrap',
    );
    fireEvent.click(screen.getByTitle('Collapse value'));
    expect(screen.getByTitle('Expand value')).toBeInTheDocument();
  });

  it('falls back to name when copying a variable without a value', async () => {
    const writeText = vi.fn(async () => {});
    Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true });
    render(<NotebookVariables liveVariables={[variables[1]]} symbols={[]} />);
    fireEvent.click(screen.getByRole('button', { name: 'Copy name' }));
    await vi.waitFor(() => expect(writeText).toHaveBeenCalledWith('name'));
  });

  it('lists symbols when there are no live variables', () => {
    render(<NotebookVariables liveVariables={[]} symbols={symbols} />);
    expect(screen.getByText('Variables & symbols')).toBeInTheDocument();
    expect(screen.getByText('main')).toBeInTheDocument();
    expect(screen.getByText('function')).toBeInTheDocument();
    expect(screen.getByText('Config')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Refresh variables' })).toBeNull();
  });
});
