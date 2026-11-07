'use client';

import type { FC, ReactNode } from 'react';
import { motion, useReducedMotion } from 'motion/react';

type RevealProps = Readonly<{
  children: ReactNode;
  className?: string;
  delay?: number;
  y?: number;
  scale?: number;
  once?: boolean;
}>;

export const Reveal: FC<RevealProps> = ({
  children,
  className,
  delay = 0,
  y = 28,
  scale = 1,
  once = true,
}) => {
  const reduce = useReducedMotion();

  if (reduce) {
    return <div className={className}>{children}</div>;
  }

  return (
    <motion.div
      className={className}
      initial={{ opacity: 0, y, scale }}
      whileInView={{ opacity: 1, y: 0, scale: 1 }}
      viewport={{ once, amount: 0.18, margin: '0px 0px -8% 0px' }}
      transition={{ duration: 0.62, delay, ease: [0.22, 1, 0.36, 1] }}>
      {children}
    </motion.div>
  );
};
