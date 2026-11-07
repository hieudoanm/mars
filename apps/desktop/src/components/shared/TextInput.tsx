import type { FC, InputHTMLAttributes, ReactNode } from 'react';
import { cn } from './cn';

export type TextInputProps = Omit<InputHTMLAttributes<HTMLInputElement>, 'size'> & {
  label?: ReactNode;
  wrapperClassName?: string;
};

const inputClasses =
  'h-7 rounded-lg border border-neutral-700 bg-neutral-800 px-2 text-xs text-neutral-100 focus:border-orange-500 focus:outline-1 focus:outline-orange-500';

const TextInput: FC<TextInputProps> = ({ label, className, wrapperClassName, ...props }) => {
  const input = (
    <input data-component='TextInput' className={cn(inputClasses, className)} {...props} />
  );

  if (!label) return input;

  return (
    <label className={cn('flex items-center gap-1.5 text-xs text-neutral-400', wrapperClassName)}>
      <span>{label}</span>
      {input}
    </label>
  );
};

TextInput.displayName = 'TextInput';

export default TextInput;
