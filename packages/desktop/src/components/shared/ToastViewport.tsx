import { type FC } from 'react';
import { LuCircleAlert, LuCircleCheck, LuInfo, LuX } from 'react-icons/lu';
import type { Toast } from '../../hooks/useToasts';
import { cn } from './cn';

const TONE_STYLES: Record<Toast['tone'], string> = {
  info: 'border-neutral-700 bg-neutral-900 text-neutral-200',
  success: 'border-emerald-800 bg-emerald-950/90 text-emerald-200',
  error: 'border-red-800 bg-red-950/90 text-red-200',
};

const TONE_ICONS: Record<Toast['tone'], typeof LuInfo> = {
  info: LuInfo,
  success: LuCircleCheck,
  error: LuCircleAlert,
};

type ToastViewportProps = {
  toasts: Toast[];
  onDismiss: (id: number) => void;
};

const ToastViewport: FC<ToastViewportProps> = ({ toasts, onDismiss }) => {
  if (toasts.length === 0) return null;
  return (
    <div className='pointer-events-none fixed right-4 bottom-4 z-50 flex w-80 flex-col gap-2'>
      {toasts.map((toast) => {
        const Icon = TONE_ICONS[toast.tone];
        return (
          <div
            key={toast.id}
            role='status'
            className={cn(
              'pointer-events-auto flex items-start gap-2 rounded-lg border px-3 py-2 text-sm shadow-lg',
              TONE_STYLES[toast.tone],
            )}>
            <Icon className='mt-0.5 h-4 w-4 shrink-0' />
            <span className='min-w-0 flex-1 break-words'>{toast.message}</span>
            <button
              type='button'
              aria-label='Dismiss notification'
              className='shrink-0 rounded p-0.5 opacity-60 hover:opacity-100'
              onClick={() => onDismiss(toast.id)}>
              <LuX className='h-3.5 w-3.5' />
            </button>
          </div>
        );
      })}
    </div>
  );
};

ToastViewport.displayName = 'ToastViewport';
export default ToastViewport;
