import { type FC } from 'react';
import type { Notebook, NotebookCell } from '../../types/types';
import type { SearchOptions } from '../../lib/textSearch';
import UnsavedChangesDialog from '../dialogs/UnsavedChangesDialog';
import TrustDialog from '../dialogs/TrustDialog';
import PromptDialog from '../dialogs/PromptDialog';
import NotebookSearchDialog from './NotebookSearchDialog';
import DependenciesDialog from './DependenciesDialog';
import CheckpointsDialog from './CheckpointsDialog';
import RenameNotebookDialog from './RenameNotebookDialog';
import { replaceInNotebookCells, serializeNotebook } from '../../lib/notebookOperations';

type PersistenceSlice = {
  pendingOpenPath: string | null;
  closeRequested: boolean;
  pendingTrustPath: string | null;
  filePath: string | null;
  setPendingOpenPath: (path: string | null) => void;
  setCloseRequested: (value: boolean) => void;
  trustAndLoad: () => Promise<void>;
  cancelTrust: () => void;
  discardAndClose: () => void;
  discardAndOpen: () => void;
  saveAndClose: () => Promise<void>;
  saveAndOpen: () => Promise<void>;
  saveNotebook: () => Promise<boolean>;
  reloadFromDisk: () => Promise<void>;
};

type EditorSlice = {
  notebook: Notebook;
  cells: NotebookCell[];
  setNotebook: (notebook: Notebook) => void;
  setSelectedId: (id: string) => void;
  setMode: (mode: 'edit' | 'command') => void;
  setDependencies: (dependencies: Record<string, string>) => void;
  renameNotebook: (title: string) => void;
};

type NotebookSessionOverlaysProps = {
  persistence: PersistenceSlice;
  editor: EditorSlice;
  pendingPrompt: string | null;
  respondToPrompt: (text: string) => Promise<void>;
  showTabCloseDialog: boolean;
  onTabClosed: () => void;
  onTabCloseCancel: () => void;
  showSearch: boolean;
  setShowSearch: (open: boolean) => void;
  showDependencies: boolean;
  setShowDependencies: (open: boolean) => void;
  showCheckpoints: boolean;
  setShowCheckpoints: (open: boolean) => void;
  showRename: boolean;
  setShowRename: (open: boolean) => void;
  setStatus: (status: string) => void;
  onRevealMatch?: (cellId: string, line: number, character: number) => void;
};

const NotebookSessionOverlays: FC<NotebookSessionOverlaysProps> = ({
  persistence,
  editor,
  pendingPrompt,
  respondToPrompt,
  showTabCloseDialog,
  onTabClosed,
  onTabCloseCancel,
  showSearch,
  setShowSearch,
  showDependencies,
  setShowDependencies,
  showCheckpoints,
  setShowCheckpoints,
  showRename,
  setShowRename,
  setStatus,
  onRevealMatch,
}) => {
  return (
    <>
      {(persistence.pendingOpenPath || persistence.closeRequested || showTabCloseDialog) && (
        <UnsavedChangesDialog
          closeRequested={persistence.closeRequested || showTabCloseDialog}
          onCancel={() => {
            persistence.setPendingOpenPath(null);
            persistence.setCloseRequested(false);
            if (showTabCloseDialog) onTabCloseCancel();
          }}
          onDiscard={() => {
            if (showTabCloseDialog) {
              onTabClosed();
            } else if (persistence.closeRequested) {
              persistence.discardAndClose();
            } else {
              persistence.discardAndOpen();
            }
          }}
          onSave={() => {
            if (showTabCloseDialog) {
              void persistence.saveNotebook().then((saved) => {
                if (saved) onTabClosed();
              });
            } else if (persistence.closeRequested) {
              void persistence.saveAndClose();
            } else {
              void persistence.saveAndOpen();
            }
          }}
        />
      )}
      {persistence.pendingTrustPath && (
        <TrustDialog
          fileName={persistence.pendingTrustPath.split('/').pop() ?? 'notebook'}
          onTrust={() => void persistence.trustAndLoad()}
          onCancel={persistence.cancelTrust}
        />
      )}
      <NotebookSearchDialog
        cells={editor.cells}
        open={showSearch}
        onClose={() => setShowSearch(false)}
        onReplaceAll={(find: string, replace: string, options: SearchOptions) => {
          const result = replaceInNotebookCells(editor.notebook, find, replace, options);
          if (result.count === 0) {
            setStatus('No occurrences found');
            return;
          }
          editor.setNotebook(result.notebook);
          setStatus(`Replaced ${result.count} occurrence(s)`);
        }}
        onSelectCell={(cellId, lineNumber, character) => {
          if (onRevealMatch) {
            onRevealMatch(cellId, lineNumber - 1, character);
            return;
          }
          editor.setSelectedId(cellId);
          editor.setMode('command');
          window.requestAnimationFrame(() => {
            document.getElementById(`cell-${cellId}`)?.scrollIntoView({ block: 'center' });
          });
        }}
      />
      <DependenciesDialog
        open={showDependencies}
        dependencies={editor.notebook.metadata.dependencies ?? {}}
        onSave={(dependencies) => editor.setDependencies(dependencies)}
        onClose={() => setShowDependencies(false)}
      />
      {pendingPrompt !== null ? (
        <PromptDialog message={pendingPrompt} onSubmit={(answer) => void respondToPrompt(answer)} />
      ) : null}
      <RenameNotebookDialog
        open={showRename}
        title={editor.notebook.metadata.title ?? 'Untitled notebook'}
        onRename={editor.renameNotebook}
        onClose={() => setShowRename(false)}
      />
      <CheckpointsDialog
        open={showCheckpoints}
        filePath={persistence.filePath}
        currentContent={serializeNotebook(editor.notebook)}
        onRestored={() => void persistence.reloadFromDisk()}
        onStatus={setStatus}
        onClose={() => setShowCheckpoints(false)}
      />
    </>
  );
};

NotebookSessionOverlays.displayName = 'NotebookSessionOverlays';
export default NotebookSessionOverlays;
