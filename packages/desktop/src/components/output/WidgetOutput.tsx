import { type FC, useEffect, useSyncExternalStore } from 'react';
import {
  ensureWidgetBridge,
  getWidget,
  sendWidgetComm,
  subscribeWidgets,
} from '../../lib/widgetStore';
import { cn, CodeBlock } from '../shared';
import { OUTPUT_SURFACE } from './outputStyles';

const asString = (value: unknown): string => (typeof value === 'string' ? value : '');
const asNumber = (value: unknown, fallback: number): number =>
  typeof value === 'number' && Number.isFinite(value) ? value : fallback;
const asBool = (value: unknown): boolean => value === true;

const updateValue = (commId: string, value: unknown) => {
  void sendWidgetComm(commId, 'comm_msg', {
    method: 'update',
    state: { value },
    buffer_paths: [],
  });
};

const fieldLabel = (state: Record<string, unknown>): string => {
  const description = asString(state.description);
  return description || asString(state._model_name).replace(/Model$/, '');
};

const WidgetControl: FC<{ commId: string; state: Record<string, unknown> }> = ({
  commId,
  state,
}) => {
  const disabled = asBool(state.disabled);
  const modelName = asString(state._model_name);
  const value = state.value;

  if (modelName === 'ButtonModel') {
    return (
      <button
        type='button'
        disabled={disabled}
        className='rounded-md border border-neutral-700 bg-neutral-800 px-3 py-1 text-sm text-neutral-200 hover:bg-neutral-700 disabled:opacity-50'
        onClick={() => void sendWidgetComm(commId, 'comm_msg', { event: 'click' })}>
        {fieldLabel(state)}
      </button>
    );
  }

  if (modelName === 'IntSliderModel' || modelName === 'FloatSliderModel') {
    return (
      <div className='flex items-center gap-3'>
        <span className='w-24 shrink-0 text-xs text-neutral-400'>{fieldLabel(state)}</span>
        <input
          type='range'
          className='min-w-0 flex-1 accent-sky-500'
          min={asNumber(state.min, 0)}
          max={asNumber(state.max, 100)}
          step={asNumber(state.step, 1)}
          disabled={disabled}
          value={asNumber(value, asNumber(state.min, 0))}
          onChange={(event) =>
            updateValue(
              commId,
              modelName === 'IntSliderModel'
                ? Number.parseInt(event.target.value, 10)
                : Number.parseFloat(event.target.value),
            )
          }
        />
        <span className='w-12 shrink-0 text-right font-mono text-xs text-neutral-300'>
          {String(value ?? '')}
        </span>
      </div>
    );
  }

  if (modelName === 'CheckboxModel') {
    return (
      <label className='flex items-center gap-2 text-sm text-neutral-200'>
        <input
          type='checkbox'
          className='accent-sky-500'
          disabled={disabled}
          checked={value === true}
          onChange={(event) => updateValue(commId, event.target.checked)}
        />
        {fieldLabel(state)}
      </label>
    );
  }

  if (
    modelName === 'DropdownModel' ||
    modelName === 'SelectModel' ||
    modelName === 'RadioButtonsModel' ||
    modelName === 'SelectionSliderModel'
  ) {
    const options = Array.isArray(state.options) ? state.options : [];
    const pairs = options.map((option) => {
      if (Array.isArray(option) && option.length >= 2) {
        return { label: String(option[0]), value: option[1] };
      }
      return { label: String(option), value: option };
    });
    return (
      <label className='flex items-center gap-2 text-sm text-neutral-200'>
        <span className='text-xs text-neutral-400'>{fieldLabel(state)}</span>
        <select
          className='rounded-md border border-neutral-700 bg-neutral-900 px-2 py-1 text-sm text-neutral-200 disabled:opacity-50'
          disabled={disabled}
          value={JSON.stringify(value ?? null)}
          onChange={(event) => {
            const chosen = pairs.find((pair) => JSON.stringify(pair.value) === event.target.value);
            if (chosen) updateValue(commId, chosen.value);
          }}>
          {pairs.map((pair, index) => (
            <option key={index} value={JSON.stringify(pair.value)}>
              {pair.label}
            </option>
          ))}
        </select>
      </label>
    );
  }

  if (modelName === 'TextInputModel' || modelName === 'TextModel') {
    return (
      <label className='flex flex-col gap-1 text-xs text-neutral-400'>
        {fieldLabel(state)}
        <input
          type='text'
          className='w-full rounded-md border border-neutral-700 bg-neutral-900 px-2 py-1 text-sm text-neutral-200 disabled:opacity-50'
          disabled={disabled}
          defaultValue={asString(value)}
          onBlur={(event) => {
            if (event.target.value !== asString(value)) updateValue(commId, event.target.value);
          }}
          onKeyDown={(event) => {
            if (event.key === 'Enter') event.currentTarget.blur();
          }}
        />
      </label>
    );
  }

  if (modelName === 'TextareaModel') {
    return (
      <label className='flex flex-col gap-1 text-xs text-neutral-400'>
        {fieldLabel(state)}
        <textarea
          className='w-full rounded-md border border-neutral-700 bg-neutral-900 px-2 py-1 font-mono text-sm text-neutral-200 disabled:opacity-50'
          rows={3}
          disabled={disabled}
          defaultValue={asString(value)}
          onBlur={(event) => {
            if (event.target.value !== asString(value)) updateValue(commId, event.target.value);
          }}
        />
      </label>
    );
  }

  if (modelName === 'IntProgressModel' || modelName === 'FloatProgressModel') {
    const max = asNumber(state.max, 100);
    const current = asNumber(value, 0);
    return (
      <div className='flex items-center gap-2'>
        <progress className='h-2 min-w-0 flex-1 accent-sky-500' max={max} value={current} />
        <span className='text-xs text-neutral-400'>
          {current}/{max}
        </span>
      </div>
    );
  }

  return (
    <CodeBlock tone='muted'>
      {`Widget ${modelName || commId} (unsupported model — state preserved)`}
    </CodeBlock>
  );
};

/// Renders a Jupyter widget referenced by `model_id` (its comm id). Widget
/// state arrives over the kernel comm channel; controls send `comm_msg`
/// updates back to the owning language runtime.
const WidgetOutput: FC<{ modelId: string }> = ({ modelId }) => {
  useEffect(() => ensureWidgetBridge(), []);
  const entry = useSyncExternalStore(subscribeWidgets, () => getWidget(modelId));
  if (!entry) {
    return (
      <div className={cn(OUTPUT_SURFACE, 'text-xs text-neutral-500')}>
        Widget state unavailable — re-run the cell to recreate it.
      </div>
    );
  }
  return (
    <div className={cn(OUTPUT_SURFACE, 'flex flex-col gap-2 py-2')} data-widget={modelId}>
      <WidgetControl commId={modelId} state={entry.state} />
    </div>
  );
};
WidgetOutput.displayName = 'WidgetOutput';
export default WidgetOutput;
