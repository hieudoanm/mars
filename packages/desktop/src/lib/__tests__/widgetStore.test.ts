import {
  applyCommMessages,
  getWidget,
  subscribeWidgets,
} from '../widgetStore';
import type { CommMessage } from '../../types/types';

const comm = (partial: Partial<CommMessage> & Pick<CommMessage, 'comm_id' | 'comm_type'>): CommMessage => ({
  data: {},
  ...partial,
});

describe('widgetStore', () => {
  it('registers widgets on comm_open and merges update state', () => {
    applyCommMessages(
      [
        comm({
          comm_id: 'w1',
          comm_type: 'comm_open',
          target_name: 'jupyter.widget',
          data: { state: { _model_name: 'IntSliderModel', value: 3, min: 0, max: 10 } },
        }),
        comm({
          comm_id: 'w1',
          comm_type: 'comm_msg',
          data: { method: 'update', state: { value: 7 } },
        }),
      ],
      'session-a',
      'python',
    );
    const widget = getWidget('w1');
    expect(widget?.state.value).toBe(7);
    expect(widget?.state._model_name).toBe('IntSliderModel');
    expect(widget?.session).toBe('session-a');
    expect(widget?.language).toBe('python');
  });

  it('applies update_states across model ids', () => {
    applyCommMessages(
      [
        comm({ comm_id: 'a', comm_type: 'comm_open', data: { state: { value: 1 } } }),
        comm({ comm_id: 'b', comm_type: 'comm_open', data: { state: { value: 2 } } }),
        comm({
          comm_id: 'a',
          comm_type: 'comm_msg',
          data: { method: 'update_states', states: { b: { value: 9 } } },
        }),
      ],
      's',
      'python',
    );
    expect(getWidget('b')?.state.value).toBe(9);
  });

  it('removes widgets on comm_close and notifies subscribers', () => {
    applyCommMessages(
      [comm({ comm_id: 'c1', comm_type: 'comm_open', data: { state: {} } })],
      's',
      'python',
    );
    const listener = vi.fn();
    const unsubscribe = subscribeWidgets(listener);
    applyCommMessages([comm({ comm_id: 'c1', comm_type: 'comm_close' })], 's', 'python');
    expect(getWidget('c1')).toBeUndefined();
    expect(listener).toHaveBeenCalled();
    unsubscribe();
  });
});
