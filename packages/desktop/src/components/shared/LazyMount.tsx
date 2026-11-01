import { type FC, type ReactNode, useEffect, useRef, useState } from 'react';

type LazyMountProps = {
  children: ReactNode;
  rootMargin?: string;
  placeholderClassName?: string;
};

/**
 * Mounts children only when the element approaches the viewport. Used to keep
 * offscreen CodeMirror instances from being created in large notebooks.
 * The placeholder reserves approximate space so the scrollbar stays usable.
 */
const LazyMount: FC<LazyMountProps> = ({
  children,
  rootMargin = '900px',
  placeholderClassName = 'min-h-28',
}) => {
  const ref = useRef<HTMLDivElement>(null);
  const [nearViewport, setNearViewport] = useState(false);

  useEffect(() => {
    const element = ref.current;
    if (!element || nearViewport) return;
    if (typeof IntersectionObserver === 'undefined') {
      setNearViewport(true);
      return;
    }
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setNearViewport(true);
          observer.disconnect();
        }
      },
      { rootMargin },
    );
    observer.observe(element);
    return () => observer.disconnect();
  }, [nearViewport, rootMargin]);

  return (
    <div ref={ref} data-component='LazyMount'>
      {nearViewport ? children : <div aria-hidden='true' className={placeholderClassName} />}
    </div>
  );
};

LazyMount.displayName = 'LazyMount';
export default LazyMount;
