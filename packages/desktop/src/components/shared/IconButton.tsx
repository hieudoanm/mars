import { type FC, type ReactNode, useId } from 'react';
import Button, { type ButtonProps } from './Button';
import { cn } from './cn';

export type IconButtonProps = Omit<ButtonProps, 'children'> & {
  label: string;
  icon: ReactNode;
};

const IconButton: FC<IconButtonProps> = ({ label, icon, className, ...props }) => {
  const tooltipId = useId();

  return (
    <span className='group relative inline-flex'>
      <Button
        size='icon'
        aria-label={label}
        aria-describedby={tooltipId}
        className={className}
        {...props}>
        {icon}
      </Button>
      <span
        id={tooltipId}
        role='tooltip'
        data-component='IconButtonTooltip'
        className={cn(
          'pointer-events-none absolute top-full left-1/2 z-20 mt-2 -translate-x-1/2 rounded-md border border-neutral-700 bg-neutral-950 px-2 py-1 text-[11px] whitespace-nowrap text-neutral-200 opacity-0 shadow-lg transition-opacity',
          'group-focus-within:opacity-100 group-hover:opacity-100',
        )}>
        {label}
      </span>
    </span>
  );
};

IconButton.displayName = 'IconButton';

export default IconButton;
