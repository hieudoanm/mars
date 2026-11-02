import { repositoryUrl } from '@/lib/site';

const sectionShell =
  'mx-auto w-[min(calc(100%_-_48px),1160px)] max-[520px]:w-[min(calc(100%_-_32px),1160px)]';

const downloads = [
  {
    platform: 'macOS',
    icon: '',
    description: 'Universal build for Apple Silicon and Intel',
    links: [{ label: 'Download .dmg', href: `${repositoryUrl}/releases` }],
  },
  {
    platform: 'Windows',
    icon: '⊞',
    description: 'x64 installer',
    links: [{ label: 'Download .msi', href: `${repositoryUrl}/releases` }],
  },
  {
    platform: 'Linux',
    icon: '🐧',
    description: 'AppImage and .deb packages',
    links: [{ label: 'Download', href: `${repositoryUrl}/releases` }],
  },
];

export default function DownloadsPage() {
  return (
    <div className='min-h-screen bg-[#0b1118] text-[#e6ebf2]'>
      <header className='mx-auto flex w-[min(calc(100%_-_48px),1160px)] items-center justify-between py-7 max-[520px]:w-[min(calc(100%_-_32px),1160px)]'>
        <a
          className='inline-flex items-center gap-2.5 text-[19px] font-[650] tracking-[-.03em]'
          href='/'
          aria-label='Mars home'>
          <img className='h-[30px] w-[30px] rounded-lg' src='/mars-logo.svg' alt='' />
          <span>Mars</span>
        </a>
        <nav className='flex gap-[30px] text-[13px] text-[#8b96a6] max-[850px]:hidden'>
          <a className='hover:text-[#e6ebf2]' href='/#why-mars'>
            Why Mars
          </a>
          <a className='hover:text-[#e6ebf2]' href='/docs'>
            Docs
          </a>
          <a className='hover:text-[#e6ebf2]' href='/downloads'>
            Downloads
          </a>
          <a className='hover:text-[#e6ebf2]' href='/#get-started'>
            Get started
          </a>
        </nav>
        <a
          className='text-[13px] font-semibold text-[#ff8c5a] hover:text-[#e6ebf2]'
          href={repositoryUrl}>
          GitHub <span aria-hidden='true'>↗</span>
        </a>
      </header>

      <main>
        <section className={`${sectionShell} py-[80px] max-[520px]:py-[60px]`}>
          <div className='mb-[60px] max-w-[600px]'>
            <div className='flex items-center gap-[9px] text-[11px] font-bold tracking-[.18em] text-[#ff8c5a] uppercase'>
              <span className='h-[7px] w-[7px] rounded-full bg-[#f06b4d] shadow-[0_0_15px_#f06b4d]' />
              Download
            </div>
            <h1 className='mt-5 mb-[18px] text-[clamp(38px,5vw,62px)] leading-[1.02] font-[620] tracking-[-.065em]'>
              Get Mars for your platform
            </h1>
            <p className='text-[18px] text-[#8b96a6]'>
              Download the desktop app or use the CLI from source.
            </p>
          </div>

          <div className='grid grid-cols-3 gap-4 max-[850px]:grid-cols-1'>
            {downloads.map((item) => (
              <article
                key={item.platform}
                className='rounded-[13px] border border-[#1e2b38] bg-[linear-gradient(145deg,#151f2b,#101720)] p-7'>
                <div className='mb-6 text-3xl'>{item.icon}</div>
                <h2 className='mb-2 text-xl tracking-[-.04em]'>{item.platform}</h2>
                <p className='mb-6 text-[13px] text-[#8b96a6]'>{item.description}</p>
                <div className='flex flex-col gap-2'>
                  {item.links.map((link) => (
                    <a
                      key={link.label}
                      className='inline-flex items-center gap-[14px] rounded-lg border border-[#f06b4d] bg-[#f06b4d] px-[18px] py-[13px] text-[13px] font-[650] text-[#fff7ed] shadow-[0_10px_30px_#f06b4d25] transition-[transform] duration-200 hover:-translate-y-0.5'
                      href={link.href}>
                      {link.label} <span aria-hidden='true'>↗</span>
                    </a>
                  ))}
                </div>
              </article>
            ))}
          </div>

          <div className='mt-12 rounded-[13px] border border-[#1e2b38] bg-[#111a25] p-7'>
            <h3 className='mb-2 text-lg'>Install via Cargo</h3>
            <p className='mb-4 text-[13px] text-[#8b96a6]'>Build and install the CLI from source</p>
            <code className='block rounded-lg border border-[#1e2b38] bg-[#0b1118] px-4 py-3 font-mono text-sm'>
              cargo install --git https://github.com/hieudoanm/mars mars-cli
            </code>
          </div>
        </section>
      </main>

      <footer
        className={`${sectionShell} flex items-center justify-between gap-5 py-[30px] text-[11px] text-[#6d7a8a] max-[520px]:flex-col max-[520px]:items-start`}>
        <a
          className='inline-flex items-center gap-2.5 text-[19px] font-[650] tracking-[-.03em] text-[#d8dde5]'
          href='/'>
          <img className='h-6 w-6 rounded-lg' src='/mars-logo.svg' alt='' />
          <span>Mars</span>
        </a>
        <span>Explore ideas. Execute code. Discover possibilities.</span>
        <span>MIT licensed · Built with Rust</span>
      </footer>
    </div>
  );
}
