import { type FC } from 'react';
import { Button, Dialog } from '../shared';

type UnsavedChangesDialogProps = {
  closeRequested: boolean;
  onCancel: () => void;
  onDiscard: () => void;
  onSave: () => void;
};

const UnsavedChangesDialog: FC<UnsavedChangesDialogProps> = ({
  closeRequested,
  onCancel,
  onDiscard,
  onSave,
}) => {
  return (
    <Dialog labelledBy='unsaved-title' component='UnsavedChangesDialog'>
      <span className='text-xs tracking-widest text-neutral-400 uppercase'>UNSAVED NOTEBOOK</span>
      <h2 id='unsaved-title' className='mt-2 text-lg font-medium tracking-tight'>
        {closeRequested ? 'Save changes before closing?' : 'Save changes before opening?'}
      </h2>
      <p className='mt-2 leading-relaxed text-neutral-400'>
        Your current notebook has unsaved changes. Choose how Project Mars should continue.
      </p>
      <div className='mt-6 flex justify-end gap-2'>
        <Button onClick={onCancel}>Cancel</Button>
        <Button variant='danger' onClick={onDiscard}>
          {closeRequested ? 'Close without saving' : 'Open without saving'}
        </Button>
        <Button variant='primary' onClick={onSave}>
          {closeRequested ? 'Save and close' : 'Save and open'}
        </Button>
      </div>
    </Dialog>
  );
};
UnsavedChangesDialog.displayName = 'UnsavedChangesDialog';
export default UnsavedChangesDialog;
