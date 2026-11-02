const sectionShell =
  'mx-auto w-[min(calc(100%_-_48px),1160px)] max-[520px]:w-[min(calc(100%_-_32px),1160px)]';
const eyebrow =
  'flex items-center gap-[9px] text-[11px] font-bold uppercase tracking-[.18em] text-orange-2';
const heading =
  'mt-5 mb-[18px] text-[clamp(38px,5vw,62px)] font-[620] leading-[1.02] tracking-[-.065em] max-[520px]:text-[42px]';

export default function BuiltDifferent() {
  return (
    <section
      className={`${sectionShell} grid grid-cols-2 items-center gap-[100px] py-[150px] max-[850px]:grid-cols-1 max-[850px]:gap-[50px] max-[850px]:py-[100px]`}
      id='built-different'>
      <div className='relative min-h-[410px] pt-[38px]'>
        <div className='relative mx-auto -mb-[2px] w-[80%] translate-x-[38px] rounded-[9px] border border-[#354556] bg-[#1b2a38] px-6 py-[18px] shadow-[0_12px_25px_#0003]'>
          <span className='text-orange mb-[5px] block font-mono text-[9px] tracking-[.16em]'>
            DESKTOP
          </span>
          <strong className='block text-[17px]'>Notebook UI</strong>
          <small className='text-[11px] text-[#738398]'>React · Tauri</small>
        </div>
        <div className='relative mx-auto -mb-[2px] w-[80%] translate-x-[18px] rounded-[9px] border border-[#354556] bg-[#182634] px-6 py-[18px] shadow-[0_12px_25px_#0003]'>
          <span className='text-orange mb-[5px] block font-mono text-[9px] tracking-[.16em]'>
            KERNEL
          </span>
          <strong className='block text-[17px]'>Typed protocol</strong>
          <small className='text-[11px] text-[#738398]'>Requests · events · state</small>
        </div>
        <div className='relative mx-auto -mb-[2px] w-[80%] rounded-[9px] border border-[#354556] bg-[#151f2d] px-6 py-[18px] shadow-[0_12px_25px_#0003]'>
          <span className='text-orange mb-[5px] block font-mono text-[9px] tracking-[.16em]'>
            RUNTIME
          </span>
          <strong className='block text-[17px]'>Rust execution</strong>
          <small className='text-[11px] text-[#738398]'>
            Isolated · persistent · resettable
          </small>
        </div>
        <div className='mt-[30px] text-[13px] leading-[1.5] text-[#7e8da0]'>
          Clear boundaries.
          <br />
          <em className='text-orange-2 font-serif text-xl'>Better experiments.</em>
        </div>
      </div>
      <div>
        <div className={eyebrow}>Built differently, on purpose</div>
        <h2 className={heading}>
          Not a web wrapper.
          <br />
          Not a black box.
        </h2>
        <p className='max-w-[480px]'>
          Mars is designed from the document model outward. The UI, kernel, protocol, renderer,
          runtime, and CLI are separate pieces you can understand, test, and extend.
        </p>
        <ul className='mt-[35px] grid list-none gap-[21px] p-0'>
          <li className='flex items-start gap-[14px]'>
            <span className='grid h-[21px] w-[21px] place-items-center rounded-full bg-[#214e3d] text-[11px] text-[#81dcae]'>
              ✓
            </span>
            <div>
              <strong className='block text-[13px]'>Rust-first foundation</strong>
              <small className='mt-1 block text-[11px] text-[#7f8da0]'>
                Core behavior is shared across desktop and CLI.
              </small>
            </div>
          </li>
          <li className='flex items-start gap-[14px]'>
            <span className='grid h-[21px] w-[21px] place-items-center rounded-full bg-[#214e3d] text-[11px] text-[#81dcae]'>
              ✓
            </span>
            <div>
              <strong className='block text-[13px]'>Reproducible execution</strong>
              <small className='mt-1 block text-[11px] text-[#7f8da0]'>
                Timeouts, lockfiles, offline, and deterministic controls.
              </small>
            </div>
          </li>
          <li className='flex items-start gap-[14px]'>
            <span className='grid h-[21px] w-[21px] place-items-center rounded-full bg-[#214e3d] text-[11px] text-[#81dcae]'>
              ✓
            </span>
            <div>
              <strong className='block text-[13px]'>Open interoperability</strong>
              <small className='mt-1 block text-[11px] text-[#7f8da0]'>
                Import and export representative Jupyter notebooks.
              </small>
            </div>
          </li>
        </ul>
      </div>
    </section>
  );
}
