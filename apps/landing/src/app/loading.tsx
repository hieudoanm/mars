export default function Loading() {
  return (
    <div className='bg-bg text-ink flex min-h-screen flex-col items-center justify-center'>
      <div className='flex flex-col items-center gap-4'>
        <img className='h-[40px] w-[40px]' src='/mars-logo.svg' alt='' />
        <p className='text-muted animate-pulse'>Loading Mars…</p>
      </div>
    </div>
  );
}
