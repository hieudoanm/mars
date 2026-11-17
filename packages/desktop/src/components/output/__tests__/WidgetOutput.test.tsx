import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import WidgetOutput from '../WidgetOutput';
import { getWidget, sendWidgetComm } from '../../../lib/widgetStore';

vi.mock('../../../lib/widgetStore', () => ({
  ensureWidgetBridge: vi.fn(),
  getWidget: vi.fn(),
  sendWidgetComm: vi.fn().mockResolvedValue(undefined),
  subscribeWidgets: vi.fn(() => () => {}),
}));

const entry = (state: Record<string, unknown>) => ({ commId: 'w1', state });

const renderWidget = (state: Record<string, unknown>) => {
  vi.mocked(getWidget).mockReturnValue(entry(state) as never);
  return render(<WidgetOutput modelId='w1' />);
};

describe('WidgetOutput', () => {
  beforeEach(() => vi.clearAllMocks());

  it('shows a fallback when widget state is unavailable', () => {
    vi.mocked(getWidget).mockReturnValue(undefined);
    render(<WidgetOutput modelId='missing' />);
    expect(screen.getByText(/Widget state unavailable/)).toBeInTheDocument();
  });

  it('renders a ButtonModel and sends a click comm', () => {
    renderWidget({ _model_name: 'ButtonModel', description: 'Run', value: null });
    fireEvent.click(screen.getByRole('button', { name: 'Run' }));
    expect(sendWidgetComm).toHaveBeenCalledWith('w1', 'comm_msg', {
      event: 'click',
    });
  });

  it('disables controls when state.disabled is set', () => {
    renderWidget({ _model_name: 'ButtonModel', description: 'Run', disabled: true });
    expect(screen.getByRole('button', { name: 'Run' })).toBeDisabled();
  });

  it('renders an IntSliderModel and sends integer updates', () => {
    renderWidget({
      _model_name: 'IntSliderModel',
      description: 'Epochs',
      min: 0,
      max: 10,
      step: 1,
      value: 3,
    });
    const slider = screen.getByRole('slider');
    fireEvent.change(slider, { target: { value: '7' } });
    expect(sendWidgetComm).toHaveBeenCalledWith('w1', 'comm_msg', {
      method: 'update',
      state: { value: 7 },
      buffer_paths: [],
    });
  });

  it('renders a FloatSliderModel and sends float updates', () => {
    renderWidget({
      _model_name: 'FloatSliderModel',
      description: 'LR',
      min: 0,
      max: 1,
      step: 0.01,
      value: 0.5,
    });
    fireEvent.change(screen.getByRole('slider'), { target: { value: '0.25' } });
    expect(sendWidgetComm).toHaveBeenCalledWith(
      'w1',
      'comm_msg',
      expect.objectContaining({ state: { value: 0.25 } }),
    );
  });

  it('renders a CheckboxModel and sends boolean updates', () => {
    renderWidget({ _model_name: 'CheckboxModel', description: 'Enable', value: true });
    const checkbox = screen.getByRole('checkbox');
    expect(checkbox).toBeChecked();
    fireEvent.click(checkbox);
    expect(sendWidgetComm).toHaveBeenCalledWith(
      'w1',
      'comm_msg',
      expect.objectContaining({ state: { value: false } }),
    );
  });

  it('renders a DropdownModel and sends the chosen option value', () => {
    renderWidget({
      _model_name: 'DropdownModel',
      description: 'Optimizer',
      options: [
        ['Adam', 'adam'],
        ['SGD', 'sgd'],
      ],
      value: 'adam',
    });
    const select = screen.getByRole('combobox');
    fireEvent.change(select, { target: { value: '"sgd"' } });
    expect(sendWidgetComm).toHaveBeenCalledWith(
      'w1',
      'comm_msg',
      expect.objectContaining({ state: { value: 'sgd' } }),
    );
  });

  it('supports tuple options and scalar options in selects', () => {
    renderWidget({
      _model_name: 'SelectModel',
      description: 'Pick',
      options: ['a', 'b'],
      value: 'a',
    });
    const select = screen.getByRole('combobox');
    fireEvent.change(select, { target: { value: '"b"' } });
    expect(sendWidgetComm).toHaveBeenCalledWith(
      'w1',
      'comm_msg',
      expect.objectContaining({ state: { value: 'b' } }),
    );
  });

  it('renders a TextInputModel and sends updates on blur', () => {
    renderWidget({ _model_name: 'TextInputModel', description: 'Name', value: 'old' });
    const input = screen.getByRole('textbox');
    fireEvent.change(input, { target: { value: 'new' } });
    fireEvent.blur(input);
    expect(sendWidgetComm).toHaveBeenCalledWith(
      'w1',
      'comm_msg',
      expect.objectContaining({ state: { value: 'new' } }),
    );
  });

  it('does not send an update when text is unchanged', () => {
    renderWidget({ _model_name: 'TextModel', description: 'Name', value: 'same' });
    const input = screen.getByRole('textbox');
    fireEvent.blur(input);
    expect(sendWidgetComm).not.toHaveBeenCalled();
  });

  it('renders a TextareaModel', () => {
    renderWidget({ _model_name: 'TextareaModel', description: 'Notes', value: 'hi' });
    const area = screen.getByRole('textbox');
    expect(area.tagName).toBe('TEXTAREA');
    fireEvent.change(area, { target: { value: 'changed' } });
    fireEvent.blur(area);
    expect(sendWidgetComm).toHaveBeenCalledWith(
      'w1',
      'comm_msg',
      expect.objectContaining({ state: { value: 'changed' } }),
    );
  });

  it('renders progress models', () => {
    const { container } = renderWidget({
      _model_name: 'IntProgressModel',
      value: 40,
      max: 100,
    });
    expect(container.querySelector('progress')).toBeInTheDocument();
    expect(screen.getByText('40/100')).toBeInTheDocument();
  });

  it('renders an unsupported-model fallback', () => {
    renderWidget({ _model_name: 'MysteryModel', value: 1 });
    expect(screen.getByText(/unsupported model/)).toBeInTheDocument();
  });
});
