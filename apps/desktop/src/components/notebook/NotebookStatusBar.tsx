import { type FC } from 'react';
import { Badge } from '../shared';

type NotebookStatusBarProps = {
  cellCount: number;
  isDirty: boolean;
};

const NotebookStatusBar: FC<NotebookStatusBarProps> = ({ cellCount, isDirty }) => {
  return (
    <footer
      data-component='NotebookStatusBar'
      className='flex gap-5 border-t border-neutral-800 bg-neutral-900 px-3 py-1.5 text-xs text-neutral-400'>
      <span>Rust 2024</span>
      <Badge dot dotTone='success'>
        Kernel connected
      </Badge>
      <span>{cellCount} cells</span>
      <Badge tone={isDirty ? 'warning' : 'success'}>
        {isDirty ? 'Unsaved changes' : 'All changes saved'}
      </Badge>
    </footer>
  );
};
NotebookStatusBar.displayName = 'NotebookStatusBar';
export default NotebookStatusBar;
