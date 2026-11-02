import { repositoryUrl } from "../lib/site";

const sectionShell =
  "mx-auto w-[min(calc(100%_-_48px),1160px)] max-[520px]:w-[min(calc(100%_-_32px),1160px)]";
const headerShell =
  "relative z-[2] mx-auto flex w-[min(calc(100%_-_48px),1240px)] items-center justify-between py-7 max-[520px]:w-[min(calc(100%_-_32px),1240px)] max-[520px]:py-5";
const heading =
  "mt-5 mb-[18px] text-[clamp(38px,5vw,62px)] font-[620] leading-[1.02] tracking-[-.065em] max-[520px]:text-[42px]";
const buttonBase =
  "inline-flex items-center gap-[14px] rounded-lg border px-[18px] py-[13px] text-[13px] font-[650] transition-[transform,border-color,background] duration-200 hover:-translate-y-0.5 hover:border-[#d4654d]";
const buttonPrimary = `${buttonBase} border-orange bg-orange text-[#fff7ed] shadow-[0_10px_30px_#f06b4d25] hover:bg-[#ff7955]`;
const buttonQuiet = `${buttonBase} border-line bg-[#141d28] text-[#c8d0db]`;
const eyebrow =
  "flex items-center gap-[9px] text-[11px] font-bold uppercase tracking-[.18em] text-orange-2";

const features = [
  {
    large: true,
    icon: "✦",
    iconClass: "",
    title: "Explore without context switching",
    body: "Mix Rust, Markdown, mathematics, tables, charts, and rich output in one document that stays readable as the idea grows.",
  },
  {
    icon: "◈",
    iconClass: "bg-[#36755d42] text-[#83d6ae]",
    title: "Isolation by default",
    body: "Notebook code runs outside the desktop process in a controlled runtime boundary. Reset, interrupt, and inspect with confidence.",
    footnote: "Process-isolated execution",
  },
  {
    icon: "▣",
    iconClass: "bg-[#8b543342] text-[#ffc07b]",
    title: "Own your documents",
    body: "The .mars format is open, JSON-based, versioned, and designed to remain useful beyond one application.",
    footnote: "Human-readable by design",
  },
];

const workflow = [
  ["01", "Write", "Capture the thought in code or Markdown."],
  ["02", "Run", "Execute Rust with persistent bindings."],
  ["03", "See", "Turn results into rich, useful output."],
  ["04", "Share", "Export, reproduce, and keep moving."],
];

const tools = [
  [
    "M",
    "",
    "CANONICAL",
    "Desktop app",
    "A polished notebook workspace with CodeMirror, rich output, autosave, and native dialogs.",
    "Explore desktop",
  ],
  [
    ">_",
    "bg-[#285541] font-mono text-base text-[#9be0bc]",
    "HEADLESS",
    "Command line",
    "Create, run, inspect, import, export, and automate notebooks anywhere Cargo runs.",
    "Read the CLI guide",
  ],
  [
    "◇",
    "bg-[#384a75] text-2xl text-[#b1caff]",
    "COMPANION",
    "VS Code",
    "Syntax support, cell navigation, preview, export, and CLI-backed notebook execution.",
    "See the extension",
  ],
];

function App() {
  return (
    <div className="relative overflow-hidden">
      <div className="pointer-events-none absolute top-20 right-[2%] h-[520px] w-[520px] rounded-full bg-[radial-gradient(circle,#d9554420_0,transparent_68%)] blur-[1px]" />
      <div className="pointer-events-none absolute top-[600px] left-[-250px] h-[550px] w-[550px] rounded-full bg-[radial-gradient(circle,#28405d30_0,transparent_70%)] blur-[1px]" />
      <header className={headerShell}>
        <a
          className="inline-flex items-center gap-2.5 text-[19px] font-[650] tracking-[-.03em]"
          href="#"
          aria-label="Mars home"
        >
          <img className="h-[30px] w-[30px] rounded-lg" src="/mars-logo.svg" alt="" />
          <span>Mars</span>
        </a>
        <nav
          className="flex gap-[30px] text-[13px] text-muted max-[850px]:hidden"
          aria-label="Primary navigation"
        >
          <a className="hover:text-ink" href="#why-mars">Why Mars</a>
          <a className="hover:text-ink" href="#workflow">Workflow</a>
          <a className="hover:text-ink" href="#built-different">Architecture</a>
          <a className="hover:text-ink" href="/docs">Docs</a>
          <a className="hover:text-ink" href="#get-started">Get started</a>
        </nav>
        <a
          className="text-[13px] font-semibold text-orange-2 hover:text-ink max-[520px]:text-[11px]"
          href={repositoryUrl}
        >
          GitHub <span aria-hidden="true">↗</span>
        </a>
      </header>

      <main>
        <section
          className={`${sectionShell} relative grid min-h-[640px] grid-cols-[1fr_.9fr] items-center gap-[50px] pt-20 pb-[110px] max-[850px]:grid-cols-1 max-[850px]:pt-[55px] max-[520px]:pt-[50px] max-[520px]:pb-[75px]`}
        >
          <div className="relative z-[1]">
            <div className={eyebrow}>
              <span className="h-[7px] w-[7px] rounded-full bg-orange shadow-[0_0_15px_#f06b4d]" /> Rust-powered interactive notebooks
            </div>
            <h1 className="mt-6 mb-[22px] max-w-[670px] text-[clamp(54px,7vw,92px)] font-[650] leading-[.96] tracking-[-.075em] max-[850px]:text-[64px] max-[520px]:text-[52px]">
              Think in cells.<br /><em className="font-serif font-normal text-orange-2">Build in Rust.</em>
            </h1>
            <p className="m-0 max-w-[535px] text-[18px] max-[520px]:text-[16px]">Mars is a focused environment for exploring ideas, executing code, and discovering possibilities - without giving up the power of a real developer tool.</p>
            <div className="my-8 flex flex-wrap gap-3">
              <a className={buttonPrimary} href="#get-started">Start exploring <span aria-hidden="true">→</span></a>
              <a className={buttonQuiet} href={repositoryUrl}>View on GitHub <span aria-hidden="true">↗</span></a>
            </div>
            <div className="flex flex-wrap gap-[18px] text-[11px] text-faint max-[520px]:grid max-[520px]:gap-2">
              <span className="flex items-center gap-1.5"><span className="font-extrabold text-mint">✓</span> Native <code>.mars</code> format</span>
              <span className="flex items-center gap-1.5"><span className="font-extrabold text-mint">✓</span> Isolated Rust execution</span>
              <span className="flex items-center gap-1.5"><span className="font-extrabold text-mint">✓</span> Jupyter import/export</span>
            </div>
          </div>
          <div className="relative grid h-[500px] place-items-center max-[850px]:h-[390px] max-[520px]:h-[330px]" aria-label="Mars notebook interface preview">
            <div className="absolute h-[190px] w-[530px] rotate-[-25deg] rounded-full border border-[#d6745840] max-[520px]:w-[360px]" />
            <div className="absolute h-[140px] w-[450px] rotate-[36deg] rounded-full border border-[#8498ad30] max-[520px]:w-[300px]" />
            <div className="relative z-[1] h-[300px] w-[300px] animate-float drop-shadow-[0_28px_45px_#0008] max-[520px]:h-[220px] max-[520px]:w-[220px]">
              <img className="h-full w-full" src="/mars-logo.svg" alt="Mars logo" />
            </div>
            <div className="absolute top-[74px] right-0 z-[2] rounded-[9px] border border-[#3b4857] bg-[#141d28e8] px-4 py-[14px] font-mono text-xs leading-[1.6] shadow-[0_18px_40px_#0005] max-[520px]:top-6 max-[520px]:right-[-7px] max-[520px]:text-[9px]">
              <span className="mb-2.5 flex gap-1">
                <i className="h-[5px] w-[5px] rounded-full bg-[#c35f4a]" />
                <i className="h-[5px] w-[5px] rounded-full bg-[#d39b55]" />
                <i className="h-[5px] w-[5px] rounded-full bg-[#6eaf83]" />
              </span>
              <code><b>let</b> ideas = <span className="text-[#f39467]">explore</span>();</code>
            </div>
            <div className="absolute bottom-[72px] left-[2px] z-[2] rounded-[9px] border border-[#3b4857] bg-[#141d28e8] px-4 py-[14px] font-mono text-xs leading-[1.6] shadow-[0_18px_40px_#0005] max-[520px]:bottom-5 max-[520px]:left-[-7px] max-[520px]:text-[9px]">
              <span className="mb-[5px] block text-[9px] tracking-[.12em] text-[#718096]">OUT [03]</span>
              <code><span className="text-[#75d6a5]">discoveries</span> = 42</code>
            </div>
            <span className="absolute top-1/4 left-[10%] text-[13px] text-[#ffb478]">✦</span>
            <span className="absolute top-[35%] right-[7%] text-[9px] text-[#ffb478]">✦</span>
            <span className="absolute right-[22%] bottom-[18%] text-[25px] text-[#ffb478]">·</span>
          </div>
        </section>

        <section className="border-y border-line bg-[#0e151f]">
          <div className={`${sectionShell} flex min-h-[74px] items-center justify-between text-[11px] uppercase tracking-[.08em] text-faint max-[520px]:block max-[520px]:py-[18px]`}>
            <span className="max-[520px]:max-w-[130px]">Made for people who think in systems</span>
            <div className="flex gap-7 font-bold text-[#77869a] max-[850px]:gap-2.5 max-[850px]:text-[9px] max-[520px]:mt-3 max-[520px]:justify-between"><span>SCIENCE</span><span>DATA</span><span>LEARNING</span><span>EXPERIMENTS</span></div>
          </div>
        </section>

        <section className={`${sectionShell} py-[145px] max-[520px]:py-[90px]`} id="why-mars">
          <div className="max-w-[590px]">
            <div className={eyebrow}>A better place for unfinished thoughts</div>
            <h2 className={heading}>From first question<br />to working <em className="font-serif font-normal text-orange-2">insight.</em></h2>
            <p className="max-w-[470px] text-base">Mars keeps the fast feedback of a notebook and brings the boundaries, portability, and clarity of a developer tool.</p>
          </div>
          <div className="mt-[55px] grid grid-cols-[1.3fr_1fr_1fr] gap-4 max-[850px]:grid-cols-1">
            {features.map((feature) => (
              <article className={`relative min-h-[320px] overflow-hidden rounded-[13px] border border-line bg-[linear-gradient(145deg,#151f2b,#101720)] p-7 ${feature.large ? "max-[850px]:min-h-[380px]" : ""}`} key={feature.title}>
                <div className={`mb-[35px] grid h-10 w-10 place-items-center rounded-[10px] bg-[#e25e4750] text-xl text-white ${feature.iconClass}`}>{feature.icon}</div>
                <h3 className="m-0 mb-3 text-xl tracking-[-.04em]">{feature.title}</h3>
                <p className="m-0 text-[13px]">{feature.body}</p>
                {feature.title === "Explore without context switching" ? (
                  <div className="absolute right-[22px] bottom-[-2px] left-[22px] rounded-t-lg border border-b-0 border-[#304052] bg-[#0d141d] px-[18px] pt-[17px] pb-3 font-mono text-[11px] leading-[1.9]">
                    <div className="mb-[5px] whitespace-nowrap text-[#c4cbd5]"># Analyze the signal</div>
                    <div className="whitespace-nowrap"><span className="inline-block w-[25px] text-[#566578]">01</span><span className="text-[#d39cff]">let</span> signal = <span className="text-[#f5986d]">load</span>();</div>
                    <div className="whitespace-nowrap"><span className="inline-block w-[25px] text-[#566578]">02</span>signal.<span className="text-[#77b7ff]">plot</span>();</div>
                    <div className="mt-[5px] border-t border-[#263341] pt-[5px] text-[#72d3a2]">↳ chart rendered in 12ms</div>
                  </div>
                ) : (
                  <div className="absolute bottom-[25px] flex items-center gap-2 text-[11px] text-[#7f8c9c]"><span className="h-1.5 w-1.5 rounded-full bg-mint" /> {feature.footnote}</div>
                )}
              </article>
            ))}
          </div>
        </section>

        <section className="border-y border-[#1e2b38] bg-[#0d141e]" id="workflow">
          <div className={`${sectionShell} py-[125px] pb-[135px] max-[520px]:py-[90px]`}>
            <div className="mx-auto max-w-[590px] text-center">
              <div className={`${eyebrow} justify-center`}>One document. Many ways forward.</div>
              <h2 className={heading}>A workflow that follows<br /><em className="font-serif font-normal text-orange-2">your curiosity.</em></h2>
            </div>
            <div className="mt-[70px] flex items-stretch max-[850px]:grid max-[850px]:grid-cols-2 max-[850px]:gap-5 max-[520px]:grid-cols-1">
              {workflow.map(([number, title, body], index) => (
                <div className="contents" key={title}>
                  <div className={`flex-1 border-t px-[22px] py-5 ${index === 0 ? "border-t-2 border-orange" : "border-[#2b3a4a]"}`}>
                    <span className="mb-8 block font-mono text-xs text-orange">{number}</span><strong className="text-lg">{title}</strong><p className="mb-0 text-xs">{body}</p>
                  </div>
                  {index < workflow.length - 1 && <div className="mt-0 w-[18px] border-t border-dashed border-[#344455] max-[850px]:hidden" />}
                </div>
              ))}
            </div>
          </div>
        </section>

        <section className={`${sectionShell} grid grid-cols-2 items-center gap-[100px] py-[150px] max-[850px]:grid-cols-1 max-[850px]:gap-[50px] max-[850px]:py-[100px]`} id="built-different">
          <div className="relative min-h-[410px] pt-[38px]">
            <div className="relative mx-auto -mb-[2px] w-[80%] translate-x-[38px] rounded-[9px] border border-[#354556] bg-[#1b2a38] px-6 py-[18px] shadow-[0_12px_25px_#0003]"><span className="mb-[5px] block font-mono text-[9px] tracking-[.16em] text-orange">DESKTOP</span><strong className="block text-[17px]">Notebook UI</strong><small className="text-[11px] text-[#738398]">React · Tauri</small></div>
            <div className="relative mx-auto -mb-[2px] w-[80%] translate-x-[18px] rounded-[9px] border border-[#354556] bg-[#182634] px-6 py-[18px] shadow-[0_12px_25px_#0003]"><span className="mb-[5px] block font-mono text-[9px] tracking-[.16em] text-orange">KERNEL</span><strong className="block text-[17px]">Typed protocol</strong><small className="text-[11px] text-[#738398]">Requests · events · state</small></div>
            <div className="relative mx-auto -mb-[2px] w-[80%] rounded-[9px] border border-[#354556] bg-[#151f2d] px-6 py-[18px] shadow-[0_12px_25px_#0003]"><span className="mb-[5px] block font-mono text-[9px] tracking-[.16em] text-orange">RUNTIME</span><strong className="block text-[17px]">Rust execution</strong><small className="text-[11px] text-[#738398]">Isolated · persistent · resettable</small></div>
            <div className="mt-[30px] text-[13px] leading-[1.5] text-[#7e8da0]">Clear boundaries.<br /><em className="font-serif text-xl text-orange-2">Better experiments.</em></div>
          </div>
          <div>
            <div className={eyebrow}>Built differently, on purpose</div>
            <h2 className={heading}>Not a web wrapper.<br />Not a black box.</h2>
            <p className="max-w-[480px]">Mars is designed from the document model outward. The UI, kernel, protocol, renderer, runtime, and CLI are separate pieces you can understand, test, and extend.</p>
            <ul className="mt-[35px] grid list-none gap-[21px] p-0">
              <li className="flex items-start gap-[14px]"><span className="grid h-[21px] w-[21px] place-items-center rounded-full bg-[#214e3d] text-[11px] text-[#81dcae]">✓</span><div><strong className="block text-[13px]">Rust-first foundation</strong><small className="mt-1 block text-[11px] text-[#7f8da0]">Core behavior is shared across desktop and CLI.</small></div></li>
              <li className="flex items-start gap-[14px]"><span className="grid h-[21px] w-[21px] place-items-center rounded-full bg-[#214e3d] text-[11px] text-[#81dcae]">✓</span><div><strong className="block text-[13px]">Reproducible execution</strong><small className="mt-1 block text-[11px] text-[#7f8da0]">Timeouts, lockfiles, offline, and deterministic controls.</small></div></li>
              <li className="flex items-start gap-[14px]"><span className="grid h-[21px] w-[21px] place-items-center rounded-full bg-[#214e3d] text-[11px] text-[#81dcae]">✓</span><div><strong className="block text-[13px]">Open interoperability</strong><small className="mt-1 block text-[11px] text-[#7f8da0]">Import and export representative Jupyter notebooks.</small></div></li>
            </ul>
          </div>
        </section>

        <section className={`${sectionShell} py-[120px] pb-[150px] max-[520px]:py-[90px]`}>
          <div className="mx-auto max-w-[590px] text-center">
            <div className={`${eyebrow} justify-center`}>Meet Mars where you work</div>
            <h2 className={heading}>One core. <em className="font-serif font-normal text-orange-2">Your choice of surface.</em></h2>
          </div>
          <div className="mt-[58px] grid grid-cols-3 gap-4 max-[850px]:grid-cols-1">
            {tools.map(([mark, markClass, tag, title, body, link]) => (
              <article className="rounded-xl border border-line bg-[#111a25] p-[25px]" key={title}>
                <div className="mb-10 flex items-center justify-between"><span className={`grid h-[38px] w-[38px] place-items-center rounded-[9px] bg-[#b9523d] font-[750] text-[#fff3e7] ${markClass}`}>{mark}</span><span className="text-[9px] font-bold tracking-[.15em] text-[#6e7c8f]">{tag}</span></div>
                <h3 className="m-0 text-xl">{title}</h3><p className="min-h-[76px] text-[13px]">{body}</p><a className="text-xs font-[650] text-orange-2" href="#get-started">{link} <span className="ml-2">→</span></a>
              </article>
            ))}
          </div>
        </section>

        <section className="relative overflow-hidden border-t border-[#283646] bg-[#151d28]" id="get-started">
          <div className="absolute right-[12%] bottom-[-230px] h-[480px] w-[480px] rounded-full bg-[radial-gradient(circle,#d8564238,transparent_66%)]" />
          <div className={`${sectionShell} relative grid grid-cols-[100px_1fr_auto] items-center gap-[30px] py-[105px] max-[850px]:grid-cols-[75px_1fr] max-[520px]:grid-cols-1 max-[520px]:gap-5 max-[520px]:py-[75px]`}>
            <img className="h-[92px] w-[92px] drop-shadow-[0_15px_25px_#0006] max-[520px]:h-[70px] max-[520px]:w-[70px]" src="/mars-file-logo.svg" alt="" />
            <div><div className={eyebrow}>The surface is waiting</div><h2 className={`${heading} mb-3`}>Give your next idea<br /><em className="font-serif font-normal text-orange-2">room to move.</em></h2><p className="m-0 text-[13px]">Open source. Rust-powered. Built for the moment before the answer.</p></div>
            <div className="flex flex-col items-stretch gap-[9px] max-[850px]:col-span-full max-[850px]:flex-row max-[520px]:col-auto max-[520px]:grid">
              <a className={`${buttonPrimary} justify-center whitespace-nowrap`} href={repositoryUrl}>View the repository <span>↗</span></a>
              <a className={`${buttonQuiet} justify-center whitespace-nowrap`} href="/docs">Read the docs <span>→</span></a>
            </div>
          </div>
        </section>
      </main>
      <footer className={`${sectionShell} flex items-center justify-between gap-5 py-[30px] text-[11px] text-[#6d7a8a] max-[520px]:flex-col max-[520px]:items-start max-[520px]:py-[25px]`}>
        <a className="inline-flex items-center gap-2.5 text-[19px] font-[650] tracking-[-.03em] text-[#d8dde5]" href="#"><img className="h-6 w-6 rounded-lg" src="/mars-logo.svg" alt="" /><span>Mars</span></a>
        <span>Explore ideas. Execute code. Discover possibilities.</span>
        <span>MIT licensed · Built with Rust</span>
      </footer>
    </div>
  );
}

export default App;
