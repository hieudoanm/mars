import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import type { EditorPreferences } from '../../../hooks/useEditorPreferences';

const {
  onTimeoutChange,
  onFontSizeChange,
  onPreferencesChange,
  onCancel,
  onRunCurrent,
  onRunAll,
  onRunAbove,
  onRunBelow,
  onClearOutputs,
  onDeleteCell,
  onAddMarkdown,
  onAddCode,
} = vi.hoisted(() => ({
  onTimeoutChange: vi.fn(),
  onFontSizeChange: vi.fn(),
  onPreferencesChange: vi.fn(),
  onCancel: vi.fn(),
  onRunCurrent: vi.fn(),
  onRunAll: vi.fn(),
  onRunAbove: vi.fn(),
  onRunBelow: vi.fn(),
  onClearOutputs: vi.fn(),
  onDeleteCell: vi.fn(),
  onAddMarkdown: vi.fn(),
  onAddCode: vi.fn(),
}));

import NotebookToolbarControls, {
  type NotebookToolbarControlsProps,
} from '../NotebookToolbarControls';

const preferences: EditorPreferences = { autocomplete: true, folding: false };

const renderControls = (overrides: Partial<NotebookToolbarControlsProps> = {}) =>
  render(
    <NotebookToolbarControls
      isExecuting={false}
      executionTimeout=''
      fontSize={14}
      preferences={preferences}
      onTimeoutChange={onTimeoutChange}
      onFontSizeChange={onFontSizeChange}
      onPreferencesChange={onPreferencesChange}
      onCancel={onCancel}
      onRunCurrent={onRunCurrent}
      onRunAll={onRunAll}
      onRunAbove={onRunAbove}
      onRunBelow={onRunBelow}
      onClearOutputs={onClearOutputs}
      canDeleteCell={true}
      onDeleteCell={onDeleteCell}
      onAddMarkdown={onAddMarkdown}
      onAddCode={onAddCode}
      {...overrides}
    />,
  );

describe('NotebookToolbarControls', () => {
  beforeEach(() => {
    onTimeoutChange.mockReset();
    onFontSizeChange.mockReset();
    onPreferencesChange.mockReset();
    onCancel.mockReset();
    onRunCurrent.mockReset();
    onRunAll.mockReset();
    onRunAbove.mockReset();
    onRunBelow.mockReset();
    onClearOutputs.mockReset();
    onDeleteCell.mockReset();
    onAddMarkdown.mockReset();
    onAddCode.mockReset();
  });

  it('renders execution and editor preference controls', () => {
    renderControls();

    expect(screen.getByLabelText('Execution timeout in milliseconds')).toHaveValue(null);
    expect(screen.getByLabelText('Editor font size')).toHaveValue(14);
    expect(screen.getByLabelText('Autocomplete')).toBeChecked();
    expect(screen.getByLabelText('Folding')).not.toBeChecked();
    expect(screen.getByRole('button', { name: 'Run current cell' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Run all (F6)' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Add Markdown' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Add Code' })).toBeInTheDocument();
    expect(screen.getByRole('group', { name: 'Execution controls' })).toBeInTheDocument();
    expect(screen.getByRole('group', { name: 'Output controls' })).toBeInTheDocument();
    expect(screen.getByRole('group', { name: 'Cell controls' })).toBeInTheDocument();
  });

  it('shows Stop only while executing', () => {
    const { rerender } = renderControls();
    expect(screen.queryByRole('button', { name: 'Stop' })).not.toBeInTheDocument();

    rerender(
      <NotebookToolbarControls
        isExecuting
        executionTimeout=''
        fontSize={14}
        preferences={preferences}
        onTimeoutChange={onTimeoutChange}
        onFontSizeChange={onFontSizeChange}
        onPreferencesChange={onPreferencesChange}
        onCancel={onCancel}
        onRunCurrent={onRunCurrent}
        onRunAll={onRunAll}
        onRunAbove={onRunAbove}
        onRunBelow={onRunBelow}
        onClearOutputs={onClearOutputs}
        canDeleteCell
        onDeleteCell={onDeleteCell}
        onAddMarkdown={onAddMarkdown}
        onAddCode={onAddCode}
      />,
    );

    fireEvent.click(screen.getByRole('button', { name: 'Stop' }));
    expect(onCancel).toHaveBeenCalledOnce();
  });

  it('routes execution and cell actions to their handlers', () => {
    renderControls();

    fireEvent.click(screen.getByRole('button', { name: 'Run current cell' }));
    fireEvent.click(screen.getByRole('button', { name: 'Run all (F6)' }));
    fireEvent.click(screen.getByRole('button', { name: 'Run above' }));
    fireEvent.click(screen.getByRole('button', { name: 'Run below' }));
    fireEvent.click(screen.getByRole('button', { name: 'Clear outputs' }));
    fireEvent.click(screen.getByRole('button', { name: 'Add Markdown' }));
    fireEvent.click(screen.getByRole('button', { name: 'Add Code' }));
    fireEvent.click(screen.getByRole('button', { name: 'Delete selected cell (D, D)' }));

    expect(onRunCurrent).toHaveBeenCalledOnce();
    expect(onRunAll).toHaveBeenCalledOnce();
    expect(onRunAbove).toHaveBeenCalledOnce();
    expect(onRunBelow).toHaveBeenCalledOnce();
    expect(onClearOutputs).toHaveBeenCalledOnce();
    expect(onAddMarkdown).toHaveBeenCalledOnce();
    expect(onAddCode).toHaveBeenCalledOnce();
    expect(onDeleteCell).toHaveBeenCalledOnce();
  });

  it('disables delete when the last cell cannot be removed', () => {
    renderControls({ canDeleteCell: false });
    expect(screen.getByRole('button', { name: 'Delete selected cell (D, D)' })).toBeDisabled();
  });

  it('exposes the component marker', () => {
    const { container } = renderControls();
    expect(
      container.querySelector('[data-component="NotebookToolbarControls"]'),
    ).toBeInTheDocument();
  });
});
