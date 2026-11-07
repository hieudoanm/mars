import { useCallback, useRef, useState } from 'react';

export type ToastTone = 'info' | 'success' | 'error';

export type Toast = {
  id: number;
  message: string;
  tone: ToastTone;
};

const MAX_TOASTS = 4;
const TTL: Record<ToastTone, number> = {
  info: 4000,
  success: 3000,
  error: 8000,
};

export const useToasts = () => {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const nextId = useRef(0);

  const dismissToast = useCallback((id: number) => {
    setToasts((current) => current.filter((toast) => toast.id !== id));
  }, []);

  const notify = useCallback(
    (message: string, tone: ToastTone = 'info') => {
      const id = (nextId.current += 1);
      setToasts((current) => [...current.slice(-(MAX_TOASTS - 1)), { id, message, tone }]);
      window.setTimeout(() => dismissToast(id), TTL[tone]);
    },
    [dismissToast],
  );

  return { toasts, notify, dismissToast };
};
