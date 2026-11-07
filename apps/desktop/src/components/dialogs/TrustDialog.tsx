import { type FC } from 'react';
import { LuShieldCheck } from 'react-icons/lu';
import { Button, Dialog } from '../shared';

type TrustDialogProps = {
  fileName: string;
  onTrust: () => void;
  onCancel: () => void;
};

const TrustDialog: FC<TrustDialogProps> = ({ fileName, onTrust, onCancel }) => (
  <Dialog labelledBy='trust-title' component='TrustDialog'>
    <div className='flex items-start gap-3'>
      <LuShieldCheck aria-hidden='true' className='mt-0.5 h-5 w-5 shrink-0 text-amber-400' />
      <div className='min-w-0'>
        <h2 id='trust-title' className='text-lg font-medium tracking-tight'>
          Trust this notebook?
        </h2>
        <p className='mt-2 leading-relaxed text-neutral-400'>
          <span className='font-medium text-neutral-200'>{fileName}</span> contains executable Rust
          code. Cells only run when you ask them to, inside an isolated subprocess — but review the
          source before trusting it.
        </p>
        <div className='mt-6 flex justify-end gap-2'>
          <Button onClick={onCancel}>Cancel</Button>
          <Button variant='primary' onClick={onTrust}>
            Trust &amp; open
          </Button>
        </div>
      </div>
    </div>
  </Dialog>
);
TrustDialog.displayName = 'TrustDialog';
export default TrustDialog;
