type NotebookStatusBarProps = {
  cellCount: number;
  isDirty: boolean;
};

export default function NotebookStatusBar({ cellCount, isDirty }: NotebookStatusBarProps) {
  return (
    <footer className='statusbar'>
      <span>Rust 2024</span>
      <span>Kernel connected</span>
      <span>{cellCount} cells</span>
      <span>{isDirty ? 'Unsaved changes' : 'All changes saved'}</span>
    </footer>
  );
}
