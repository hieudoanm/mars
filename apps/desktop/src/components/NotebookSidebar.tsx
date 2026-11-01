import type { NotebookCell } from '../lib/types';

type NotebookSidebarProps = {
  cells: NotebookCell[];
  selectedId: string;
  filePath: string | null;
  isDirty: boolean;
  onSelect: (id: string) => void;
};

export default function NotebookSidebar({
  cells,
  selectedId,
  filePath,
  isDirty,
  onSelect,
}: NotebookSidebarProps) {
  return (
    <aside className='sidebar'>
      <div className='sidebar-heading'>Notebook</div>
      <div className='file-card'>
        <img className='file-icon' src='/branding/mars-file-logo.svg' alt='' />
        <div>
          <strong>
            {filePath?.split('/').pop() ?? 'Untitled.mars'}
            {isDirty ? ' •' : ''}
          </strong>
          <small>{isDirty ? 'Unsaved changes' : 'Saved notebook'}</small>
        </div>
      </div>
      <div className='sidebar-heading'>Cells</div>
      {cells.map((cell, index) => (
        <button
          className={`cell-nav ${cell.id === selectedId ? 'selected' : ''}`}
          key={cell.id}
          onClick={() => onSelect(cell.id)}>
          <span>{String(index + 1).padStart(2, '0')}</span>
          {cell.cell_type}
        </button>
      ))}
    </aside>
  );
}
