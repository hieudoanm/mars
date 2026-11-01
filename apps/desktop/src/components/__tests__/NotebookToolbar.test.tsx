import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import type { EditorPreferences } from '../../hooks/useEditorPreferences';

const {
  onTimeoutChange,
  onFontSizeChange,
  onPreferencesChange,
  onCancel,
  onRunAll,
  onRunAbove,
  onRunBelow,
  onClearOutputs,
  onAddMarkdown,
  onAddCode,
} = vi.hoisted(() => ({
  onTimeoutChange: vi.fn(),
  onFontSizeChange: vi.fn(),
  onPreferencesChange: vi.fn(),
  onCancel: vi.fn(),
  onRunAll: vi.fn(),
  onRunAbove: vi.fn(),
  onRunBelow: vi.fn(),
  onClearOutputs: vi.fn(),
  onAddMarkdown: vi.fn(),
  onAddCode: vi.fn(),
}));

import NotebookToolbar from '../NotebookToolbar';

const preferences: EditorPreferences = { autocomplete: true, folding: true };

describe('NotebookToolbar', () => {
  beforeEach(() => {
    onTimeoutChange.mockReset();
    onFontSizeChange.mockReset();
    onPreferencesChange.mockReset();
    onCancel.mockReset();
    onRunAll.mockReset();
    onRunAbove.mockReset();
    onRunBelow.mockReset();
    onClearOutputs.mockReset();
    onAddMarkdown.mockReset();
    onAddCode.mockReset();
  });

  it('renders breadcrumb, title, and dirty indicator', () => {
    render(
      <NotebookToolbar
        fileName='experiment.mars'
        title='My Notebook'
        isDirty
        isExecuting={false}
        executionTimeout=''
        fontSize={14}
        preferences={preferences}
        onTimeoutChange={onTimeoutChange}
        onFontSizeChange={onFontSizeChange}
        onPreferencesChange={onPreferencesChange}
        onCancel={onCancel}
        onRunAll={onRunAll}
        onRunAbove={onRunAbove}
        onRunBelow={onRunBelow}
        onClearOutputs={onClearOutputs}
        onAddMarkdown={onAddMarkdown}
        onAddCode={onAddCode}
      />,
    );
    expect(screen.getByText(/Workspace \/ experiment\.mars/)).toBeInTheDocument();
    expect(screen.getByText(/• unsaved/)).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'My Notebook' })).toBeInTheDocument();
  });

  it('renders clean state without dirty indicator', () => {
    render(
      <NotebookToolbar
        fileName='experiment.mars'
        title='My Notebook'
        isDirty={false}
        isExecuting={false}
        executionTimeout=''
        fontSize={14}
        preferences={preferences}
        onTimeoutChange={onTimeoutChange}
        onFontSizeChange={onFontSizeChange}
        onPreferencesChange={onPreferencesChange}
        onCancel={onCancel}
        onRunAll={onRunAll}
        onRunAbove={onRunAbove}
        onRunBelow={onRunBelow}
        onClearOutputs={onClearOutputs}
        onAddMarkdown={onAddMarkdown}
        onAddCode={onAddCode}
      />,
    );
    expect(screen.queryByText('• unsaved')).not.toBeInTheDocument();
  });

  it('renders execution timeout input', () => {
    render(
      <NotebookToolbar
        fileName='experiment.mars'
        title='My Notebook'
        isDirty={false}
        isExecuting={false}
        executionTimeout=''
        fontSize={14}
        preferences={preferences}
        onTimeoutChange={onTimeoutChange}
        onFontSizeChange={onFontSizeChange}
        onPreferencesChange={onPreferencesChange}
        onCancel={onCancel}
        onRunAll={onRunAll}
        onRunAbove={onRunAbove}
        onRunBelow={onRunBelow}
        onClearOutputs={onClearOutputs}
        onAddMarkdown={onAddMarkdown}
        onAddCode={onAddCode}
      />,
    );
    expect(screen.getByLabelText('Execution timeout in milliseconds')).toBeInTheDocument();
    expect(
      screen.getByLabelText('Execution timeout in milliseconds'),
    ).toHaveValue(null);
  });

  it('renders Stop button only when executing', () => {
    render(
      <NotebookToolbar
        fileName='experiment.mars'
        title='My Notebook'
        isDirty={false}
        isExecuting={false}
        executionTimeout=''
        fontSize={14}
        preferences={preferences}
        onTimeoutChange={onTimeoutChange}
        onFontSizeChange={onFontSizeChange}
        onPreferencesChange={onPreferencesChange}
        onCancel={onCancel}
        onRunAll={onRunAll}
        onRunAbove={onRunAbove}
        onRunBelow={onRunBelow}
        onClearOutputs={onClearOutputs}
        onAddMarkdown={onAddMarkdown}
        onAddCode={onAddCode}
      />,
    );
    expect(screen.queryByRole('button', { name: 'Stop' })).not.toBeInTheDocument();
    render(
      <NotebookToolbar
        fileName='experiment.mars'
        title='My Notebook'
        isDirty={false}
        isExecuting
        executionTimeout=''
        fontSize={14}
        preferences={preferences}
        onTimeoutChange={onTimeoutChange}
        onFontSizeChange={onFontSizeChange}
        onPreferencesChange={onPreferencesChange}
        onCancel={onCancel}
        onRunAll={onRunAll}
        onRunAbove={onRunAbove}
        onRunBelow={onRunBelow}
        onClearOutputs={onClearOutputs}
        onAddMarkdown={onAddMarkdown}
        onAddCode={onAddCode}
      />,
    );
    expect(screen.getByRole('button', { name: 'Stop' })).toBeInTheDocument();
  });

  it('calls onRunAll when Run all is clicked', () => {
    render(
      <NotebookToolbar
        fileName='experiment.mars'
        title='My Notebook'
        isDirty={false}
        isExecuting={false}
        executionTimeout=''
        fontSize={14}
        preferences={preferences}
        onTimeoutChange={onTimeoutChange}
        onFontSizeChange={onFontSizeChange}
        onPreferencesChange={onPreferencesChange}
        onCancel={onCancel}
        onRunAll={onRunAll}
        onRunAbove={onRunAbove}
        onRunBelow={onRunBelow}
        onClearOutputs={onClearOutputs}
        onAddMarkdown={onAddMarkdown}
        onAddCode={onAddCode}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: /Run all/ }));
    expect(onRunAll).toHaveBeenCalledOnce();
  });

  it('calls onRunAbove when Run above is clicked', () => {
    render(
      <NotebookToolbar
        fileName='experiment.mars'
        title='My Notebook'
        isDirty={false}
        isExecuting={false}
        executionTimeout=''
        fontSize={14}
        preferences={preferences}
        onTimeoutChange={onTimeoutChange}
        onFontSizeChange={onFontSizeChange}
        onPreferencesChange={onPreferencesChange}
        onCancel={onCancel}
        onRunAll={onRunAll}
        onRunAbove={onRunAbove}
        onRunBelow={onRunBelow}
        onClearOutputs={onClearOutputs}
        onAddMarkdown={onAddMarkdown}
        onAddCode={onAddCode}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Run above' }));
    expect(onRunAbove).toHaveBeenCalledOnce();
  });

  it('calls onRunBelow when Run below is clicked', () => {
    render(
      <NotebookToolbar
        fileName='experiment.mars'
        title='My Notebook'
        isDirty={false}
        isExecuting={false}
        executionTimeout=''
        fontSize={14}
        preferences={preferences}
        onTimeoutChange={onTimeoutChange}
        onFontSizeChange={onFontSizeChange}
        onPreferencesChange={onPreferencesChange}
        onCancel={onCancel}
        onRunAll={onRunAll}
        onRunAbove={onRunAbove}
        onRunBelow={onRunBelow}
        onClearOutputs={onClearOutputs}
        onAddMarkdown={onAddMarkdown}
        onAddCode={onAddCode}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Run below' }));
    expect(onRunBelow).toHaveBeenCalledOnce();
  });

  it('calls onClearOutputs when Clear outputs is clicked', () => {
    render(
      <NotebookToolbar
        fileName='experiment.mars'
        title='My Notebook'
        isDirty={false}
        isExecuting={false}
        executionTimeout=''
        fontSize={14}
        preferences={preferences}
        onTimeoutChange={onTimeoutChange}
        onFontSizeChange={onFontSizeChange}
        onPreferencesChange={onPreferencesChange}
        onCancel={onCancel}
        onRunAll={onRunAll}
        onRunAbove={onRunAbove}
        onRunBelow={onRunBelow}
        onClearOutputs={onClearOutputs}
        onAddMarkdown={onAddMarkdown}
        onAddCode={onAddCode}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Clear outputs' }));
    expect(onClearOutputs).toHaveBeenCalledOnce();
  });

  it('calls onAddMarkdown when + Markdown is clicked', () => {
    render(
      <NotebookToolbar
        fileName='experiment.mars'
        title='My Notebook'
        isDirty={false}
        isExecuting={false}
        executionTimeout=''
        fontSize={14}
        preferences={preferences}
        onTimeoutChange={onTimeoutChange}
        onFontSizeChange={onFontSizeChange}
        onPreferencesChange={onPreferencesChange}
        onCancel={onCancel}
        onRunAll={onRunAll}
        onRunAbove={onRunAbove}
        onRunBelow={onRunBelow}
        onClearOutputs={onClearOutputs}
        onAddMarkdown={onAddMarkdown}
        onAddCode={onAddCode}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: '+ Markdown' }));
    expect(onAddMarkdown).toHaveBeenCalledOnce();
  });

  it('calls onAddCode when + Code is clicked', () => {
    render(
      <NotebookToolbar
        fileName='experiment.mars'
        title='My Notebook'
        isDirty={false}
        isExecuting={false}
        executionTimeout=''
        fontSize={14}
        preferences={preferences}
        onTimeoutChange={onTimeoutChange}
        onFontSizeChange={onFontSizeChange}
        onPreferencesChange={onPreferencesChange}
        onCancel={onCancel}
        onRunAll={onRunAll}
        onRunAbove={onRunAbove}
        onRunBelow={onRunBelow}
        onClearOutputs={onClearOutputs}
        onAddMarkdown={onAddMarkdown}
        onAddCode={onAddCode}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: '+ Code' }));
    expect(onAddCode).toHaveBeenCalledOnce();
  });

  it('calls onCancel when Stop is clicked during execution', () => {
    render(
      <NotebookToolbar
        fileName='experiment.mars'
        title='My Notebook'
        isDirty={false}
        isExecuting
        executionTimeout=''
        fontSize={14}
        preferences={preferences}
        onTimeoutChange={onTimeoutChange}
        onFontSizeChange={onFontSizeChange}
        onPreferencesChange={onPreferencesChange}
        onCancel={onCancel}
        onRunAll={onRunAll}
        onRunAbove={onRunAbove}
        onRunBelow={onRunBelow}
        onClearOutputs={onClearOutputs}
        onAddMarkdown={onAddMarkdown}
        onAddCode={onAddCode}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Stop' }));
    expect(onCancel).toHaveBeenCalledOnce();
  });

  it('updates font size input', () => {
    render(
      <NotebookToolbar
        fileName='experiment.mars'
        title='My Notebook'
        isDirty={false}
        isExecuting={false}
        executionTimeout=''
        fontSize={14}
        preferences={preferences}
        onTimeoutChange={onTimeoutChange}
        onFontSizeChange={onFontSizeChange}
        onPreferencesChange={onPreferencesChange}
        onCancel={onCancel}
        onRunAll={onRunAll}
        onRunAbove={onRunAbove}
        onRunBelow={onRunBelow}
        onClearOutputs={onClearOutputs}
        onAddMarkdown={onAddMarkdown}
        onAddCode={onAddCode}
      />,
    );
    const input = screen.getByLabelText('Editor font size');
    expect(input).toHaveValue(14);
    fireEvent.change(input, { target: { value: '18' } });
    expect(onFontSizeChange).toHaveBeenCalledWith(18);
  });

  it('toggles autocomplete preference', () => {
    render(
      <NotebookToolbar
        fileName='experiment.mars'
        title='My Notebook'
        isDirty={false}
        isExecuting={false}
        executionTimeout=''
        fontSize={14}
        preferences={{ autocomplete: true, folding: false }}
        onTimeoutChange={onTimeoutChange}
        onFontSizeChange={onFontSizeChange}
        onPreferencesChange={onPreferencesChange}
        onCancel={onCancel}
        onRunAll={onRunAll}
        onRunAbove={onRunAbove}
        onRunBelow={onRunBelow}
        onClearOutputs={onClearOutputs}
        onAddMarkdown={onAddMarkdown}
        onAddCode={onAddCode}
      />,
    );
    const input = screen.getByLabelText(/Autocomplete/i);
    expect(input).toBeChecked();
    fireEvent.click(input);
    expect(onPreferencesChange).toHaveBeenCalledWith(expect.any(Function));
  });

  it('toggles folding preference', () => {
    render(
      <NotebookToolbar
        fileName='experiment.mars'
        title='My Notebook'
        isDirty={false}
        isExecuting={false}
        executionTimeout=''
        fontSize={14}
        preferences={{ autocomplete: false, folding: true }}
        onTimeoutChange={onTimeoutChange}
        onFontSizeChange={onFontSizeChange}
        onPreferencesChange={onPreferencesChange}
        onCancel={onCancel}
        onRunAll={onRunAll}
        onRunAbove={onRunAbove}
        onRunBelow={onRunBelow}
        onClearOutputs={onClearOutputs}
        onAddMarkdown={onAddMarkdown}
        onAddCode={onAddCode}
      />,
    );
    const input = screen.getByLabelText(/Folding/i);
    expect(input).toBeChecked();
    fireEvent.click(input);
    expect(onPreferencesChange).toHaveBeenCalledWith(expect.any(Function));
  });
});
