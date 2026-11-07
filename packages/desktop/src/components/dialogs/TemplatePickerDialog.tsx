import { type FC } from 'react';
import { LuFilePlus2, LuX } from 'react-icons/lu';
import { NOTEBOOK_TEMPLATES, type NotebookTemplate } from '../../lib/notebookTemplates';
import { Button, Dialog, IconButton } from '../shared';

type TemplatePickerDialogProps = {
  open: boolean;
  onSelect: (template: NotebookTemplate) => void;
  onClose: () => void;
};

const TemplatePickerDialog: FC<TemplatePickerDialogProps> = ({ open, onSelect, onClose }) => {
  if (!open) return null;
  return (
    <Dialog labelledBy='template-picker-title' component='TemplatePickerDialog'>
      <div className='flex items-start justify-between gap-3'>
        <div className='flex items-center gap-2'>
          <LuFilePlus2 aria-hidden='true' className='h-5 w-5 text-orange-400' />
          <h2 id='template-picker-title' className='text-lg font-medium tracking-tight'>
            New notebook from template
          </h2>
        </div>
        <IconButton
          label='Close'
          icon={<LuX aria-hidden='true' className='h-4 w-4' />}
          onClick={onClose}
        />
      </div>
      <div className='mt-4 flex flex-col gap-2'>
        {NOTEBOOK_TEMPLATES.map((template) => (
          <button
            key={template.id}
            type='button'
            className='rounded-lg border border-neutral-700 bg-neutral-800/60 px-3 py-2.5 text-left transition-colors hover:border-orange-800 hover:bg-neutral-800'
            onClick={() => onSelect(template)}>
            <span className='block text-sm font-medium text-neutral-100'>{template.name}</span>
            <span className='mt-0.5 block text-xs text-neutral-500'>{template.description}</span>
          </button>
        ))}
      </div>
      <div className='mt-4 flex justify-end'>
        <Button variant='ghost' size='sm' onClick={onClose}>
          Cancel
        </Button>
      </div>
    </Dialog>
  );
};

TemplatePickerDialog.displayName = 'TemplatePickerDialog';

export default TemplatePickerDialog;
