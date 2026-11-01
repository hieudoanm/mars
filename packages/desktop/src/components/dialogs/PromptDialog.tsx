import { type FC, useState } from 'react';
import { LuMessageCircleQuestion } from 'react-icons/lu';
import { Button, Dialog, TextInput } from '../shared';

type PromptDialogProps = {
  message: string;
  onSubmit: (answer: string) => void;
};

const PromptDialog: FC<PromptDialogProps> = ({ message, onSubmit }) => {
  const [answer, setAnswer] = useState('');

  const submit = () => {
    onSubmit(answer);
    setAnswer('');
  };

  return (
    <Dialog labelledBy='kernel-prompt-title' component='PromptDialog'>
      <div className='flex items-start gap-3'>
        <LuMessageCircleQuestion
          aria-hidden='true'
          className='mt-0.5 h-5 w-5 shrink-0 text-orange-400'
        />
        <div className='min-w-0 flex-1'>
          <h2 id='kernel-prompt-title' className='text-lg font-medium tracking-tight'>
            Kernel is waiting for input
          </h2>
          <p className='mt-2 leading-relaxed whitespace-pre-wrap text-neutral-300'>{message}</p>
          <form
            className='mt-4 flex items-center gap-2'
            onSubmit={(event) => {
              event.preventDefault();
              submit();
            }}>
            <TextInput
              autoFocus
              aria-label='Prompt answer'
              value={answer}
              onChange={(event) => setAnswer(event.target.value)}
              className='w-full'
            />
            <Button type='submit' variant='primary' size='sm'>
              Send
            </Button>
          </form>
        </div>
      </div>
    </Dialog>
  );
};

PromptDialog.displayName = 'PromptDialog';

export default PromptDialog;
