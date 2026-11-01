import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import ToastViewport from '../ToastViewport';
import type { Toast } from '../../../hooks/useToasts';

describe('ToastViewport', () => {
  it('renders toasts and dismisses on click', () => {
    const onDismiss = vi.fn();
    const toasts: Toast[] = [
      { id: 1, message: 'Save failed', tone: 'error' },
      { id: 2, message: 'Saved', tone: 'success' },
    ];
    render(<ToastViewport toasts={toasts} onDismiss={onDismiss} />);
    expect(screen.getByText('Save failed')).toBeInTheDocument();
    fireEvent.click(screen.getAllByLabelText('Dismiss notification')[0]);
    expect(onDismiss).toHaveBeenCalledWith(1);
  });

  it('renders nothing when empty', () => {
    const { container } = render(<ToastViewport toasts={[]} onDismiss={vi.fn()} />);
    expect(container).toBeEmptyDOMElement();
  });
});
