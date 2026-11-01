import { type FC } from 'react';
import NotebookSidebar, { type NotebookProblem } from './NotebookSidebar';
import type { NotebookCell, KernelVariable } from '../../types/types';
import type { LspDefinitionTarget } from '../../lib/kernelService';

type NotebookExplorerProps = {
  open: boolean;
  cells: NotebookCell[];
  selectedId: string;
  filePath: string | null;
  isDirty: boolean;
  onSelectCell: (id: string) => void;
  onRenameFile: (fileName: string) => Promise<boolean>;
  onOpenSearch: () => void;
  onOpenDependencies: () => void;
  onOpenCheckpoints: () => void;
  liveVariables: KernelVariable[];
  onRefreshVariables: () => void;
  problems: NotebookProblem[];
  onRevealLine: (target: LspDefinitionTarget) => void;
  setCommandMode: () => void;
};

const NotebookExplorer: FC<NotebookExplorerProps> = ({
  open,
  cells,
  selectedId,
  filePath,
  isDirty,
  onSelectCell,
  onRenameFile,
  onOpenSearch,
  onOpenDependencies,
  onOpenCheckpoints,
  liveVariables,
  onRefreshVariables,
  problems,
  onRevealLine,
  setCommandMode,
}) => (
  <NotebookSidebar
    open={open}
    cells={cells}
    selectedId={selectedId}
    filePath={filePath}
    isDirty={isDirty}
    onSelect={(id) => {
      onSelectCell(id);
      setCommandMode();
      window.requestAnimationFrame(() => {
        document.getElementById(`cell-${id}`)?.scrollIntoView({ block: 'center' });
      });
    }}
    onRenameFile={onRenameFile}
    onRevealHeading={(cellId, line) => onRevealLine({ cellId, line, character: 0 })}
    onOpenSearch={onOpenSearch}
    onOpenDependencies={onOpenDependencies}
    onOpenCheckpoints={onOpenCheckpoints}
    liveVariables={liveVariables}
    onRefreshVariables={onRefreshVariables}
    problems={problems}
    onSelectProblem={(problem) => {
      onSelectCell(problem.cellId);
      setCommandMode();
      onRevealLine({ cellId: problem.cellId, line: problem.line - 1, character: 0 });
    }}
  />
);

NotebookExplorer.displayName = 'NotebookExplorer';
export default NotebookExplorer;
