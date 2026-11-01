type UnsavedChangesDialogProps = {
  closeRequested: boolean;
  onCancel: () => void;
  onDiscard: () => void;
  onSave: () => void;
};

export default function UnsavedChangesDialog({
  closeRequested,
  onCancel,
  onDiscard,
  onSave,
}: UnsavedChangesDialogProps) {
  return (
    <div className='modal-backdrop' role='presentation'>
      <section
        className='confirm-modal'
        role='dialog'
        aria-modal='true'
        aria-labelledby='unsaved-title'>
        <span className='eyebrow'>UNSAVED NOTEBOOK</span>
        <h2 id='unsaved-title'>
          {closeRequested ? 'Save changes before closing?' : 'Save changes before opening?'}
        </h2>
        <p>Your current notebook has unsaved changes. Choose how Mars should continue.</p>
        <div className='modal-actions'>
          <button onClick={onCancel}>Cancel</button>
          <button onClick={onDiscard}>
            {closeRequested ? 'Close without saving' : 'Open without saving'}
          </button>
          <button className='primary' onClick={onSave}>
            {closeRequested ? 'Save and close' : 'Save and open'}
          </button>
        </div>
      </section>
    </div>
  );
}
