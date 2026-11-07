import { type FC } from 'react';
import { LuFilePlus, LuFolderOpen, LuLayoutTemplate, LuNotebookPen } from 'react-icons/lu';
import { Button } from '../shared';
import NotebookHeaderBrand from '../header/NotebookHeaderBrand';

type WelcomeViewProps = {
  recents: string[];
  onOpenFile: () => void;
  onOpenFolder: () => void;
  onNewNotebook: () => void;
  onNewTemplate: () => void;
  onOpenRecent: (path: string) => void;
};

const fileName = (path: string) => path.split('/').pop() ?? path;

const WelcomeView: FC<WelcomeViewProps> = ({
  recents,
  onOpenFile,
  onOpenFolder,
  onNewNotebook,
  onNewTemplate,
  onOpenRecent,
}) => (
  <div
    data-component='WelcomeView'
    className='flex h-full flex-col items-center justify-center gap-8 overflow-y-auto px-6 py-10'>
    <div className='flex flex-col items-center gap-3 text-center'>
      <NotebookHeaderBrand />
      <p className='max-w-md text-sm leading-6 text-neutral-400'>
        Explore ideas. Execute code. Discover possibilities.
      </p>
    </div>
    <div className='grid w-full max-w-lg grid-cols-2 gap-3'>
      <Button variant='primary' onClick={onOpenFile}>
        <LuNotebookPen aria-hidden='true' className='h-4 w-4' />
        Open notebook…
      </Button>
      <Button onClick={onOpenFolder}>
        <LuFolderOpen aria-hidden='true' className='h-4 w-4' />
        Open folder…
      </Button>
      <Button onClick={onNewNotebook}>
        <LuFilePlus aria-hidden='true' className='h-4 w-4' />
        New notebook
      </Button>
      <Button onClick={onNewTemplate}>
        <LuLayoutTemplate aria-hidden='true' className='h-4 w-4' />
        From template…
      </Button>
    </div>
    <p className='text-xs text-neutral-600'>
      New notebooks start with the interactive getting-started tour. Drag a{' '}
      <code className='rounded bg-neutral-800 px-1'>.mars</code> or{' '}
      <code className='rounded bg-neutral-800 px-1'>.ipynb</code> file anywhere to open it.
    </p>
    {recents.length > 0 ? (
      <div className='w-full max-w-lg'>
        <h2 className='mb-2 text-xs tracking-widest text-neutral-500 uppercase'>Recent</h2>
        <ul className='space-y-1'>
          {recents.slice(0, 8).map((path) => (
            <li key={path}>
              <button
                type='button'
                className='flex w-full items-center gap-2 rounded-md px-3 py-1.5 text-left text-sm text-neutral-400 hover:bg-neutral-800 hover:text-neutral-200'
                title={path}
                onClick={() => onOpenRecent(path)}>
                <LuNotebookPen
                  aria-hidden='true'
                  className='h-3.5 w-3.5 shrink-0 text-orange-500'
                />
                <span className='truncate'>{fileName(path)}</span>
                <span className='ml-auto truncate text-xs text-neutral-600'>{path}</span>
              </button>
            </li>
          ))}
        </ul>
      </div>
    ) : null}
  </div>
);

WelcomeView.displayName = 'WelcomeView';
export default WelcomeView;
