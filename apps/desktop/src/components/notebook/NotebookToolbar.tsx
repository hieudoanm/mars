import { type FC } from 'react';
import NotebookToolbarControls, {
  type NotebookToolbarControlsProps,
} from './NotebookToolbarControls';
import NotebookToolbarHeading, { type NotebookToolbarHeadingProps } from './NotebookToolbarHeading';

export type NotebookToolbarProps = NotebookToolbarHeadingProps & NotebookToolbarControlsProps;

const NotebookToolbar: FC<NotebookToolbarProps> = ({
  fileName,
  isDirty,
  isExecuting,
  executionTimeout,
  fontSize,
  onTimeoutChange,
  onFontSizeChange,
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
    <div data-component='NotebookToolbar' className='mb-8 flex flex-col gap-4'>
      <NotebookToolbarHeading fileName={fileName} isDirty={isDirty} />
      <NotebookToolbarControls
        isExecuting={isExecuting}
        executionTimeout={executionTimeout}
        fontSize={fontSize}
        onTimeoutChange={onTimeoutChange}
        onFontSizeChange={onFontSizeChange}
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
