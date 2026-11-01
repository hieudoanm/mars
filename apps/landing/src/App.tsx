const repositoryUrl = "https://github.com/hieudoanm/mars";

const features = [
  {
    className: "feature-card-large",
    icon: "✦",
    iconClass: "icon-spark",
    title: "Explore without context switching",
    body: "Mix Rust, Markdown, mathematics, tables, charts, and rich output in one document that stays readable as the idea grows.",
  },
  {
    icon: "◈",
    iconClass: "icon-shield",
    title: "Isolation by default",
    body: "Notebook code runs outside the desktop process in a controlled runtime boundary. Reset, interrupt, and inspect with confidence.",
    footnote: "Process-isolated execution",
  },
  {
    icon: "▣",
    iconClass: "icon-box",
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
  ["M", "", "CANONICAL", "Desktop app", "A polished notebook workspace with CodeMirror, rich output, autosave, and native dialogs.", "Explore desktop"],
  [">_", "terminal-mark", "HEADLESS", "Command line", "Create, run, inspect, import, export, and automate notebooks anywhere Cargo runs.", "Read the CLI guide"],
  ["◇", "vscode-mark", "COMPANION", "VS Code", "Syntax support, cell navigation, preview, export, and CLI-backed notebook execution.", "See the extension"],
];

function App() {
  return (
    <div className="app">
      <div className="page-glow page-glow-one" />
      <div className="page-glow page-glow-two" />
      <header className="site-header">
        <a className="brand" href="#" aria-label="Mars home">
          <img src="/mars-logo.svg" alt="" />
          <span>Mars</span>
        </a>
        <nav className="site-nav" aria-label="Primary navigation">
          <a href="#why-mars">Why Mars</a>
          <a href="#workflow">Workflow</a>
          <a href="#built-different">Architecture</a>
          <a href="#get-started">Get started</a>
        </nav>
        <a className="header-link" href={repositoryUrl}>GitHub <span aria-hidden="true">↗</span></a>
      </header>

      <main>
        <section className="hero section-shell">
          <div className="hero-copy">
            <div className="eyebrow"><span className="eyebrow-dot" /> Rust-powered interactive notebooks</div>
            <h1>Think in cells.<br /><em>Build in Rust.</em></h1>
            <p className="hero-lede">Mars is a focused environment for exploring ideas, executing code, and discovering possibilities - without giving up the power of a real developer tool.</p>
            <div className="hero-actions">
              <a className="button button-primary" href="#get-started">Start exploring <span aria-hidden="true">→</span></a>
              <a className="button button-quiet" href={repositoryUrl}>View on GitHub <span aria-hidden="true">↗</span></a>
            </div>
            <div className="hero-proof">
              <span className="proof-item"><span className="check">✓</span> Native <code>.mars</code> format</span>
              <span className="proof-item"><span className="check">✓</span> Isolated Rust execution</span>
              <span className="proof-item"><span className="check">✓</span> Jupyter import/export</span>
            </div>
          </div>
          <div className="hero-visual" aria-label="Mars notebook interface preview">
            <div className="orbit orbit-one" />
            <div className="orbit orbit-two" />
            <div className="planet-card"><img src="/mars-logo.svg" alt="Mars logo" /></div>
            <div className="code-float code-float-top">
              <span className="window-dots"><i /><i /><i /></span>
              <code><b>let</b> ideas = <span className="code-orange">explore</span>();</code>
            </div>
            <div className="code-float code-float-bottom">
              <span className="output-label">OUT [03]</span>
              <code><span className="code-green">discoveries</span> = 42</code>
            </div>
            <span className="star star-one">✦</span><span className="star star-two">✦</span><span className="star star-three">·</span>
          </div>
        </section>

        <section className="signal-strip">
          <div className="section-shell signal-inner">
            <span>Made for people who think in systems</span>
            <div className="signal-words"><span>SCIENCE</span><span>DATA</span><span>LEARNING</span><span>EXPERIMENTS</span></div>
          </div>
        </section>

        <section className="section-shell feature-section" id="why-mars">
          <div className="section-intro">
            <div className="eyebrow">A better place for unfinished thoughts</div>
            <h2>From first question<br />to working <em>insight.</em></h2>
            <p>Mars keeps the fast feedback of a notebook and brings the boundaries, portability, and clarity of a developer tool.</p>
          </div>
          <div className="feature-grid">
            {features.map((feature) => (
              <article className={`feature-card ${feature.className ?? ""}`} key={feature.title}>
                <div className={`feature-icon ${feature.iconClass}`}>{feature.icon}</div>
                <h3>{feature.title}</h3>
                <p>{feature.body}</p>
                {feature.title === "Explore without context switching" ? (
                  <div className="mini-notebook">
                    <div className="mini-line mini-heading"># Analyze the signal</div>
                    <div className="mini-line"><span className="line-number">01</span><span className="syntax-purple">let</span> signal = <span className="syntax-orange">load</span>();</div>
                    <div className="mini-line"><span className="line-number">02</span>signal.<span className="syntax-blue">plot</span>();</div>
                    <div className="mini-result">↳ chart rendered in 12ms</div>
                  </div>
                ) : (
                  <div className="feature-footnote"><span className="tiny-dot" /> {feature.footnote}</div>
                )}
              </article>
            ))}
          </div>
        </section>

        <section className="dark-section" id="workflow">
          <div className="section-shell workflow-section">
            <div className="section-intro centered">
              <div className="eyebrow">One document. Many ways forward.</div>
              <h2>A workflow that follows<br /><em>your curiosity.</em></h2>
            </div>
            <div className="workflow-steps">
              {workflow.map(([number, title, body], index) => (
                <div className="workflow-group" key={title}>
                  <div className={`workflow-step ${index === 0 ? "active" : ""}`}>
                    <span>{number}</span><strong>{title}</strong><p>{body}</p>
                  </div>
                  {index < workflow.length - 1 && <div className="workflow-connector" />}
                </div>
              ))}
            </div>
          </div>
        </section>

        <section className="section-shell architecture-section" id="built-different">
          <div className="architecture-visual">
            <div className="stack-layer stack-ui"><span>DESKTOP</span><strong>Notebook UI</strong><small>React · Tauri</small></div>
            <div className="stack-layer stack-kernel"><span>KERNEL</span><strong>Typed protocol</strong><small>Requests · events · state</small></div>
            <div className="stack-layer stack-runtime"><span>RUNTIME</span><strong>Rust execution</strong><small>Isolated · persistent · resettable</small></div>
            <div className="stack-caption">Clear boundaries.<br /><em>Better experiments.</em></div>
          </div>
          <div className="architecture-copy">
            <div className="eyebrow">Built differently, on purpose</div>
            <h2>Not a web wrapper.<br />Not a black box.</h2>
            <p>Mars is designed from the document model outward. The UI, kernel, protocol, renderer, runtime, and CLI are separate pieces you can understand, test, and extend.</p>
            <ul className="check-list">
              <li><span>✓</span><div><strong>Rust-first foundation</strong><small>Core behavior is shared across desktop and CLI.</small></div></li>
              <li><span>✓</span><div><strong>Reproducible execution</strong><small>Timeouts, lockfiles, offline, and deterministic controls.</small></div></li>
              <li><span>✓</span><div><strong>Open interoperability</strong><small>Import and export representative Jupyter notebooks.</small></div></li>
            </ul>
          </div>
        </section>

        <section className="section-shell tools-section">
          <div className="section-intro centered">
            <div className="eyebrow">Meet Mars where you work</div>
            <h2>One core. <em>Your choice of surface.</em></h2>
          </div>
          <div className="tools-grid">
            {tools.map(([mark, markClass, tag, title, body, link]) => (
              <article className="tool-card" key={title}>
                <div className="tool-card-top"><span className={`tool-mark ${markClass}`}>{mark}</span><span className="tool-tag">{tag}</span></div>
                <h3>{title}</h3><p>{body}</p><a href="#get-started">{link} <span>→</span></a>
              </article>
            ))}
          </div>
        </section>

        <section className="cta-section" id="get-started">
          <div className="cta-orb" />
          <div className="section-shell cta-inner">
            <img src="/mars-file-logo.svg" alt="" className="cta-logo" />
            <div><div className="eyebrow">The surface is waiting</div><h2>Give your next idea<br /><em>room to move.</em></h2><p>Open source. Rust-powered. Built for the moment before the answer.</p></div>
            <div className="cta-actions">
              <a className="button button-primary" href={repositoryUrl}>View the repository <span>↗</span></a>
              <a className="button button-quiet" href={`${repositoryUrl}/blob/main/README.md`}>Read the docs <span>→</span></a>
            </div>
          </div>
        </section>
      </main>
      <footer className="site-footer section-shell">
        <a className="brand" href="#"><img src="/mars-logo.svg" alt="" /><span>Mars</span></a>
        <span>Explore ideas. Execute code. Discover possibilities.</span>
        <span>MIT licensed · Built with Rust</span>
      </footer>
    </div>
  );
}

export default App;
