type AppHeaderProps = {
  status: string;
  mode: 'command' | 'edit';
  onOpen: () => void;
  onSave: () => void;
  onExport: () => void;
  onRestart: () => void;
};

export default function AppHeader({
  status,
  mode,
  onOpen,
  onSave,
  onExport,
  onRestart,
}: AppHeaderProps) {
  return (
    <header className='titlebar'>
      <div>
        <img className='brand-mark' src='/branding/mars-logo.svg' alt='Mars' />
        <span className='eyebrow'>MARS NOTEBOOK</span>
        <h1>Explore ideas.</h1>
      </div>
      <div className='titlebar-actions'>
        <span className='status-dot' />
        <span>
          {status} · {mode === 'command' ? 'Command mode' : 'Edit mode'}
        </span>
        <button onClick={onOpen}>Open</button>
        <button onClick={onSave}>Save</button>
        <button onClick={onExport}>Export Markdown</button>
        <button onClick={onRestart}>Restart runtime</button>
      </div>
    </header>
  );
}
