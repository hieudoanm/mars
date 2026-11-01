import { type FC } from 'react';
import NotebookToolbarControls, {
  type NotebookToolbarControlsProps,
} from './NotebookToolbarControls';
import NotebookToolbarHeading, { type NotebookToolbarHeadingProps } from './NotebookToolbarHeading';

export type NotebookToolbarProps = NotebookToolbarHeadingProps & NotebookToolbarControlsProps;

const NotebookToolbar: FC<NotebookToolbarProps> = ({
  fileName,
  title,
  isDirty,
  onRename,
  isExecuting,
  executionTimeout,
  fontSize,
  preferences,
  onTimeoutChange,
  onFontSizeChange,
  onPreferencesChange,
  onCancel,
  onRunCurrent,
  onRunAll,
  onRunAbove,
  onRunBelow,
  onClearOutputs,
  canDeleteCell,
  onDeleteCell,
  onAddMarkdown,
  onAddCode,
  onAddRaw,
  onCollapseAllCells,
  onExpandAllCells,
}) => {
  return (
    <div
      data-component='NotebookToolbar'
      className='mb-10 flex flex-wrap items-end justify-between gap-6'>
      <NotebookToolbarHeading
        fileName={fileName}
        title={title}
        isDirty={isDirty}
        onRename={onRename}
      />
      <NotebookToolbarControls
        isExecuting={isExecuting}
        executionTimeout={executionTimeout}
        fontSize={fontSize}
        preferences={preferences}
        onTimeoutChange={onTimeoutChange}
        onFontSizeChange={onFontSizeChange}
        onPreferencesChange={onPreferencesChange}
        onCancel={onCancel}
        onRunCurrent={onRunCurrent}
        onRunAll={onRunAll}
        onRunAbove={onRunAbove}
        onRunBelow={onRunBelow}
        onClearOutputs={onClearOutputs}
        canDeleteCell={canDeleteCell}
        onDeleteCell={onDeleteCell}
        onAddMarkdown={onAddMarkdown}
        onAddCode={onAddCode}
        onAddRaw={onAddRaw}
        onCollapseAllCells={onCollapseAllCells}
        onExpandAllCells={onExpandAllCells}
      />
    </div>
  );
};

NotebookToolbar.displayName = 'NotebookToolbar';

export default NotebookToolbar;
