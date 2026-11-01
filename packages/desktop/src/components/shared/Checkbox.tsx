import type { FC, InputHTMLAttributes, ReactNode } from 'react';
import { cn } from './cn';

export type CheckboxProps = Omit<InputHTMLAttributes<HTMLInputElement>, 'type'> & {
  label: ReactNode;
  wrapperClassName?: string;
};

const Checkbox: FC<CheckboxProps> = ({ label, className, wrapperClassName, ...props }) => (
  <label className={cn('flex items-center gap-1.5 text-xs text-neutral-400', wrapperClassName)}>
    <input
      data-component='Checkbox'
      type='checkbox'
      className={cn('checkbox checkbox-sm checkbox-primary', className)}
      {...props}
    />
    <span>{label}</span>
  </label>
);

Checkbox.displayName = 'Checkbox';

export default Checkbox;
