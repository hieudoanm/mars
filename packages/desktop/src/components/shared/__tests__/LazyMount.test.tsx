import { act, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import LazyMount from '../LazyMount';

describe('LazyMount', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('mounts immediately when IntersectionObserver is unavailable', () => {
    vi.stubGlobal('IntersectionObserver', undefined);
    render(
      <LazyMount>
        <div>heavy child</div>
      </LazyMount>,
    );
    expect(screen.getByText('heavy child')).toBeInTheDocument();
  });

  it('mounts children once the element intersects', () => {
    let callback: IntersectionObserverCallback | undefined;
    const observe = vi.fn();
    const disconnect = vi.fn();
    class FakeIO {
      constructor(cb: IntersectionObserverCallback) {
        callback = cb;
      }
      observe = observe;
      unobserve = vi.fn();
      disconnect = disconnect;
      takeRecords = vi.fn(() => []);
      root = null;
      rootMargin = '';
      thresholds = [];
    }
    vi.stubGlobal('IntersectionObserver', FakeIO);
    render(
      <LazyMount>
        <div>lazy child</div>
      </LazyMount>,
    );
    expect(screen.queryByText('lazy child')).toBeNull();
    expect(observe).toHaveBeenCalled();
    act(() => {
      callback?.(
        [{ isIntersecting: true } as IntersectionObserverEntry],
        {} as IntersectionObserver,
      );
    });
    expect(screen.getByText('lazy child')).toBeInTheDocument();
    expect(disconnect).toHaveBeenCalled();
  });

  it('stays unmounted while not intersecting and cleans up on unmount', () => {
    let callback: IntersectionObserverCallback | undefined;
    const disconnect = vi.fn();
    class FakeIO {
      constructor(cb: IntersectionObserverCallback) {
        callback = cb;
      }
      observe = vi.fn();
      unobserve = vi.fn();
      disconnect = disconnect;
      takeRecords = vi.fn(() => []);
      root = null;
      rootMargin = '';
      thresholds = [];
    }
    vi.stubGlobal('IntersectionObserver', FakeIO);
    const { unmount } = render(
      <LazyMount rootMargin='100px'>
        <div>lazy child</div>
      </LazyMount>,
    );
    act(() => {
      callback?.(
        [{ isIntersecting: false } as IntersectionObserverEntry],
        {} as IntersectionObserver,
      );
    });
    expect(screen.queryByText('lazy child')).toBeNull();
    unmount();
    expect(disconnect).toHaveBeenCalled();
  });
});
