// All page sections for the MALD site

// ---------- Reveal hook ----------
function useReveal() {
  const ref = React.useRef(null);
  React.useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const io = new IntersectionObserver(
      (entries) => {
        entries.forEach((e) => {
          if (e.isIntersecting) {
            e.target.classList.add('in');
            io.unobserve(e.target);
          }
        });
      },
      { threshold: 0.08, rootMargin: '0px 0px -40px 0px' }
    );
    io.observe(el);
    return () => io.disconnect();
  }, []);
  return ref;
}

// ---------- Copy button ----------
function CopyButton({ text, size = 12 }) {
  const [copied, setCopied] = React.useState(false);
  const onClick = (e) => {
    e.stopPropagation();
    navigator.clipboard?.writeText(text).catch(() => {});
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };
  return (
    <button
      onClick={onClick}
      aria-label="Copy to clipboard"
      style={{
        background: 'transparent',
        border: 'none',
        padding: 0,
        color: copied ? 'var(--accent)' : 'var(--text-dim)',
        display: 'inline-flex',
        alignItems: 'center',
        justifyContent: 'center',
        transition: 'color 150ms ease-out',
      }}
      onMouseEnter={(e) => { if (!copied) e.currentTarget.style.color = 'var(--text)'; }}
      onMouseLeave={(e) => { if (!copied) e.currentTarget.style.color = 'var(--text-dim)'; }}
    >
      {copied ? <IconCheck size={size} /> : <IconCopy size={size} />}
    </button>
  );
}

// ---------- Section label ----------
function SectionLabel({ children }) {
  return (
    <div style={{
      color: 'var(--text-dim)',
      fontSize: 12,
      lineHeight: '20px',
      letterSpacing: '0.05em',
      marginBottom: 24,
    }}>
      {children}
    </div>
  );
}

// ---------- Header ----------
function Header({ accent }) {
  const [scrolled, setScrolled] = React.useState(false);
  React.useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 8);
    window.addEventListener('scroll', onScroll, { passive: true });
    onScroll();
    return () => window.removeEventListener('scroll', onScroll);
  }, []);

  return (
    <header
      className={scrolled ? 'header-blur' : ''}
      style={{
        position: 'sticky',
        top: 0,
        zIndex: 50,
        transition: 'background 200ms ease-out',
      }}
    >
      <div style={{
        maxWidth: 1200,
        margin: '0 auto',
        padding: '20px 32px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
      }}>
        <a href="#top" style={{
          color: accent,
          fontSize: 14,
          fontWeight: 600,
          letterSpacing: '0.05em',
        }}>MALD</a>
        <nav style={{ display: 'flex', alignItems: 'center', gap: 16, fontSize: 12 }}>
          <a href="#docs" style={{ color: 'var(--text-muted)' }} className="nav-link">docs</a>
          <span style={{ color: 'var(--text-dim)' }}>·</span>
          <a href="#github" style={{ color: 'var(--text-muted)' }} className="nav-link">github</a>
          <span style={{ color: 'var(--text-dim)' }}>·</span>
          <a
            href="#install"
            style={{
              color: accent,
              border: `1px solid ${accent}`,
              padding: '6px 14px',
              borderRadius: 4,
              transition: 'background 150ms ease-out',
            }}
            onMouseEnter={(e) => e.currentTarget.style.background = 'rgba(143,175,143,0.08)'}
            onMouseLeave={(e) => e.currentTarget.style.background = 'transparent'}
          >install</a>
        </nav>
      </div>
    </header>
  );
}

// ---------- Hero ----------
function Hero({ accent, shaderIntensity, headline, subline }) {
  const [copied, setCopied] = React.useState(false);
  const onCopy = () => {
    navigator.clipboard?.writeText('scoop install mald').catch(() => {});
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  const pillStyle = {
    border: '1px solid var(--border-strong)',
    color: 'var(--text-muted)',
    fontSize: 11,
    lineHeight: '16px',
    padding: '6px 12px',
    borderRadius: 4,
    background: 'rgba(17,20,22,0.4)',
    backdropFilter: 'blur(4px)',
  };

  return (
    <section id="top" style={{
      position: 'relative',
      minHeight: '100vh',
      display: 'flex',
      flexDirection: 'column',
      overflow: 'hidden',
    }}>
      <ShaderBackground intensity={shaderIntensity} />
      <div style={{
        position: 'relative',
        zIndex: 10,
        maxWidth: 1200,
        margin: '0 auto',
        padding: '0 32px',
        width: '100%',
        flex: 1,
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'center',
        paddingTop: 96,
        paddingBottom: 96,
      }}>
        {/* Pills row */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 32, flexWrap: 'wrap' }}>
          <span style={pillStyle}>local-first</span>
          <span style={{ color: 'var(--text-dim)', fontSize: 11 }}>·</span>
          <span style={pillStyle}>privacy-by-default</span>
          <span style={{ color: 'var(--text-dim)', fontSize: 11 }}>·</span>
          <span style={pillStyle}>terminal-native</span>
        </div>

        {/* H1 */}
        <h1 style={{
          margin: 0,
          fontSize: 'clamp(36px, 5.6vw, 56px)',
          lineHeight: 1.14,
          fontWeight: 600,
          color: 'var(--text)',
          letterSpacing: '-0.01em',
          maxWidth: 900,
          marginBottom: 24,
        }}>
          {headline}
        </h1>

        {/* Sub-line */}
        <p style={{
          margin: 0,
          fontSize: 14,
          lineHeight: '24px',
          color: 'var(--text-muted)',
          maxWidth: 480,
          marginBottom: 32,
        }}>
          {subline}
        </p>

        {/* CTA row */}
        <div style={{ display: 'flex', gap: 8, alignItems: 'center', flexWrap: 'wrap', marginBottom: 12 }}>
          <a
            href="#install"
            style={{
              display: 'inline-flex',
              alignItems: 'center',
              gap: 6,
              color: accent,
              border: `1px solid ${accent}`,
              background: 'var(--bg-panel-alt)',
              padding: '8px 16px',
              borderRadius: 4,
              fontSize: 12,
              transition: 'background 150ms ease-out',
            }}
            onMouseEnter={(e) => e.currentTarget.style.background = 'var(--bg-surface)'}
            onMouseLeave={(e) => e.currentTarget.style.background = 'var(--bg-panel-alt)'}
          >Install</a>
          <a
            href="#github"
            style={{
              display: 'inline-flex',
              alignItems: 'center',
              gap: 8,
              color: 'var(--text-muted)',
              border: '1px solid var(--border-strong)',
              padding: '8px 16px',
              borderRadius: 4,
              fontSize: 12,
              transition: 'color 150ms ease-out, border-color 150ms ease-out',
            }}
            onMouseEnter={(e) => { e.currentTarget.style.color = 'var(--text)'; e.currentTarget.style.borderColor = 'var(--text-muted)'; }}
            onMouseLeave={(e) => { e.currentTarget.style.color = 'var(--text-muted)'; e.currentTarget.style.borderColor = 'var(--border-strong)'; }}
          >
            <IconDiamond size={12} />
            View on GitHub
          </a>
        </div>

        {/* Inline copy pill */}
        <div
          onClick={onCopy}
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: 10,
            background: 'var(--bg-panel-alt)',
            border: '1px solid var(--border)',
            padding: '6px 12px',
            borderRadius: 4,
            fontSize: 11,
            color: 'var(--text)',
            width: 'fit-content',
            cursor: 'pointer',
            userSelect: 'none',
            transition: 'border-color 150ms ease-out',
          }}
          onMouseEnter={(e) => e.currentTarget.style.borderColor = 'var(--border-strong)'}
          onMouseLeave={(e) => e.currentTarget.style.borderColor = 'var(--border)'}
        >
          <span style={{ color: 'var(--accent)' }}>$</span>
          <span>scoop install mald</span>
          <span style={{ color: copied ? 'var(--accent)' : 'var(--text-dim)', display: 'inline-flex' }}>
            {copied ? <IconCheck size={12} /> : <IconCopy size={12} />}
          </span>
        </div>
      </div>

      {/* Scroll indicator */}
      <div style={{
        position: 'relative',
        zIndex: 10,
        textAlign: 'center',
        paddingBottom: 48,
        color: 'var(--text-dim)',
        fontSize: 11,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        gap: 8,
      }}>
        <span>scroll</span>
        <IconArrowDown size={10} />
      </div>
    </section>
  );
}

// ---------- Glyph holder ----------
function GlyphHolder({ children, accent }) {
  return (
    <div style={{
      width: 24,
      height: 24,
      border: `1px solid ${accent}`,
      borderRadius: 4,
      display: 'inline-flex',
      alignItems: 'center',
      justifyContent: 'center',
      color: accent,
      background: 'rgba(143,175,143,0.06)',
      marginBottom: 16,
    }}>
      {children}
    </div>
  );
}

// ---------- What It Is ----------
function WhatItIs({ accent }) {
  const ref = useReveal();
  const cards = [
    { Icon: IconFileText, title: 'Local files', body: 'Plain markdown stays readable, portable, and yours. No proprietary workspace format.' },
    { Icon: IconSparkles, title: 'Local AI', body: 'Ollama-backed retrieval augmented generation with inspectable, cited answers.' },
    { Icon: IconShield, title: 'Sovereign defaults', body: 'No telemetry, no account, no cloud dependency, and no forced sync layer.' },
  ];
  return (
    <section ref={ref} className="reveal" style={{ maxWidth: 1200, margin: '0 auto', padding: '96px 32px 0' }}>
      <SectionLabel>// what it is</SectionLabel>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 16 }}>
        {cards.map((c, i) => (
          <div key={i} style={{
            background: 'var(--bg-panel)',
            border: '1px solid var(--border)',
            borderRadius: 8,
            padding: 24,
          }}>
            <GlyphHolder accent={accent}><c.Icon size={12} color={accent} /></GlyphHolder>
            <h3 style={{ margin: 0, fontSize: 14, lineHeight: '22px', fontWeight: 500, color: 'var(--text)', marginBottom: 8 }}>{c.title}</h3>
            <p style={{ margin: 0, fontSize: 13, lineHeight: '22px', color: 'var(--text-muted)' }}>{c.body}</p>
          </div>
        ))}
      </div>
    </section>
  );
}

// ---------- Faux text bar ----------
function Bar({ width = '90%', height = 10 }) {
  return (
    <div style={{
      width,
      height,
      background: 'var(--bg-surface)',
      border: '1px solid var(--border)',
      borderRadius: 4,
    }} />
  );
}

// ---------- Three Surfaces ----------
function ThreeSurfaces({ accent }) {
  const ref = useReveal();
  const [tab, setTab] = React.useState('GUI');
  const tabs = ['GUI', 'TUI', 'CLI'];
  return (
    <section ref={ref} className="reveal" style={{ maxWidth: 1200, margin: '0 auto', padding: '96px 32px 0' }}>
      <SectionLabel>// three surfaces</SectionLabel>

      <div style={{ display: 'flex', gap: 8, marginBottom: 24 }}>
        {tabs.map((t) => {
          const active = tab === t;
          return (
            <button
              key={t}
              onClick={() => setTab(t)}
              style={{
                padding: '6px 16px',
                borderRadius: 999,
                border: active ? `1px solid ${accent}` : '1px solid var(--border)',
                background: active ? 'rgba(143,175,143,0.18)' : 'var(--bg-panel)',
                color: active ? accent : 'var(--text-muted)',
                fontSize: 12,
                fontFamily: 'inherit',
                transition: 'all 150ms ease-out',
              }}
            >
              {t}
            </button>
          );
        })}
      </div>

      <FauxWindow surface={tab} accent={accent} />
    </section>
  );
}

// ---------- Faux window ----------
function FauxWindow({ surface, accent }) {
  return (
    <div style={{
      background: 'var(--bg-panel)',
      border: '1px solid var(--border)',
      borderRadius: 8,
      overflow: 'hidden',
    }}>
      {/* Title bar */}
      <div style={{
        height: 32,
        background: 'var(--bg-panel-alt)',
        borderBottom: '1px solid var(--border)',
        display: 'flex',
        alignItems: 'center',
        padding: '0 12px',
        position: 'relative',
      }}>
        <div style={{ display: 'flex', gap: 6 }}>
          <span style={{ width: 8, height: 8, borderRadius: '50%', background: '#C67878', display: 'inline-block' }} />
          <span style={{ width: 8, height: 8, borderRadius: '50%', background: '#BFA76A', display: 'inline-block' }} />
          <span style={{ width: 8, height: 8, borderRadius: '50%', background: accent, display: 'inline-block' }} />
        </div>
        <span style={{
          position: 'absolute',
          left: '50%',
          top: '50%',
          transform: 'translate(-50%, -50%)',
          color: 'var(--text-dim)',
          fontSize: 11,
        }}>~/mald/vault/01_Projects/MALD</span>
      </div>

      {/* Body */}
      {surface === 'GUI' && <GuiBody accent={accent} />}
      {surface === 'TUI' && <TuiBody accent={accent} />}
      {surface === 'CLI' && <CliBody accent={accent} />}
    </div>
  );
}

function ColHeader({ children }) {
  return (
    <div style={{
      color: 'var(--text-dim)',
      fontSize: 11,
      letterSpacing: '0.1em',
      marginBottom: 12,
    }}>{children}</div>
  );
}

function GuiBody({ accent }) {
  const files = [
    { name: '00_Inbox', active: false, indent: 0 },
    { name: '01_Projects', active: false, indent: 0 },
    { name: 'MALD', active: false, indent: 1 },
    { name: 'architecture.md', active: true, indent: 2 },
    { name: 'tasks.md', active: false, indent: 2 },
    { name: '03_Resources', active: false, indent: 0 },
    { name: 'papers', active: false, indent: 1 },
    { name: 'journal', active: false, indent: 1 },
  ];
  return (
    <div style={{
      display: 'grid',
      gridTemplateColumns: '200px 1fr 220px',
      gap: 16,
      padding: 16,
    }}>
      {/* Vault */}
      <div>
        <ColHeader>VAULT</ColHeader>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
          {files.map((f, i) => (
            <div key={i} style={{
              display: 'flex',
              alignItems: 'center',
              gap: 8,
              padding: '4px 6px',
              paddingLeft: 6 + f.indent * 10,
              borderRadius: 4,
              background: f.active ? 'var(--bg-surface)' : 'transparent',
              fontSize: 12,
              color: f.active ? 'var(--text)' : 'var(--text-muted)',
            }}>
              <span style={{
                width: 6, height: 6,
                borderRadius: '50%',
                background: f.active ? accent : 'transparent',
                border: f.active ? 'none' : '1px solid var(--text-dim)',
                flexShrink: 0,
              }} />
              <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{f.name}</span>
            </div>
          ))}
        </div>
        <div style={{ marginTop: 24, height: 4, background: 'var(--bg-surface)', borderRadius: 2, overflow: 'hidden' }}>
          <div style={{ width: '60%', height: '100%', background: accent }} />
        </div>
      </div>

      {/* Markdown preview */}
      <div>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 12 }}>
          <span style={{ color: 'var(--text-dim)', fontSize: 11 }}>markdown preview</span>
          <span style={{
            border: `1px solid ${accent}`,
            color: accent,
            fontSize: 11,
            padding: '2px 10px',
            borderRadius: 999,
          }}>local index</span>
        </div>

        <h4 style={{ margin: 0, fontSize: 18, lineHeight: '28px', fontWeight: 500, color: 'var(--text)', marginBottom: 12 }}>## Project Roadmap</h4>

        <div style={{ display: 'flex', flexDirection: 'column', gap: 8, marginBottom: 20 }}>
          <Bar width="80%" />
          <Bar width="95%" />
          <Bar width="70%" />
          <Bar width="85%" />
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 12, marginBottom: 20 }}>
          {['CLI', 'TUI', 'GUI'].map((label) => (
            <div key={label} style={{
              background: 'var(--bg-panel-alt)',
              border: '1px solid var(--border)',
              borderRadius: 8,
              padding: 12,
              display: 'flex', flexDirection: 'column', gap: 6,
            }}>
              <span style={{ color: 'var(--text)', fontSize: 12 }}>{label}</span>
              <Bar width="90%" height={6} />
              <Bar width="70%" height={6} />
              <Bar width="80%" height={6} />
            </div>
          ))}
        </div>

        <div style={{
          background: 'var(--bg-panel-alt)',
          border: '1px solid var(--border)',
          borderRadius: 8,
          padding: 16,
        }}>
          <div style={{ color: 'var(--text-dim)', fontSize: 11, letterSpacing: '0.1em', marginBottom: 12 }}>CITED ANSWER</div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            <Bar width="92%" />
            <Bar width="78%" />
            <Bar width="88%" />
            <Bar width="60%" />
          </div>
        </div>
      </div>

      {/* Context */}
      <div>
        <ColHeader>CONTEXT</ColHeader>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          {[0.91, 0.92, 0.93].map((s, i) => (
            <div key={i} style={{
              display: 'flex',
              alignItems: 'center',
              gap: 8,
              background: 'var(--bg-panel-alt)',
              border: '1px solid var(--border)',
              borderRadius: 4,
              padding: '8px 12px',
              fontSize: 11,
            }}>
              <span style={{ color: accent }}>[{i + 1}]</span>
              <span style={{ color: 'var(--text)', flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>source.md</span>
              <span style={{ color: 'var(--accent-warm)' }}>{s.toFixed(2)}</span>
            </div>
          ))}
        </div>
        <div style={{ marginTop: 24, height: 4, background: 'var(--bg-surface)', borderRadius: 2, overflow: 'hidden' }}>
          <div style={{ width: '80%', height: '100%', background: accent }} />
        </div>
        <div style={{ marginTop: 16, display: 'flex', flexDirection: 'column', gap: 8 }}>
          <Bar width="70%" />
          <Bar width="90%" />
          <Bar width="55%" />
        </div>
      </div>
    </div>
  );
}

function TuiBody({ accent }) {
  // Ratatui-style TUI rendered with CSS panels — no fragile box-drawing chars.
  const files = [
    { name: '00_Inbox', active: false, indent: 0 },
    { name: '01_Projects', active: false, indent: 0 },
    { name: 'MALD', active: false, indent: 1 },
    { name: 'architecture.md', active: true, indent: 2 },
    { name: 'tasks.md', active: false, indent: 2 },
    { name: '03_Resources', active: false, indent: 0 },
    { name: 'papers', active: false, indent: 1 },
  ];
  const previewLines = [
    { t: '## Project Roadmap', c: 'var(--text)', w: 500 },
    { t: '', c: 'var(--text-muted)' },
    { t: '- [x] Ship HNSW search', c: 'var(--text-muted)' },
    { t: '- [x] Wire ollama backend', c: 'var(--text-muted)' },
    { t: '- [ ] File watcher daemon', c: 'var(--text-muted)' },
    { t: '- [ ] Multi-space workspaces', c: 'var(--text-muted)' },
    { t: '', c: 'var(--text-muted)' },
    { t: 'See [[architecture]] for details.', c: 'var(--text-dim)' },
  ];
  const sources = [
    { i: 1, name: 'hnsw.md', score: '0.91' },
    { i: 2, name: 'arch.md', score: '0.92' },
    { i: 3, name: 'rag.md', score: '0.93' },
  ];

  const cellHeader = (label) => (
    <div style={{
      padding: '4px 10px',
      fontSize: 11,
      color: 'var(--text-dim)',
      letterSpacing: '0.08em',
      borderBottom: '1px solid var(--border-strong)',
      background: 'rgba(143,175,143,0.04)',
    }}>{label}</div>
  );

  const cellStyle = {
    border: '1px solid var(--border-strong)',
    borderRadius: 4,
    background: '#0B0D0E',
    overflow: 'hidden',
    display: 'flex',
    flexDirection: 'column',
    minHeight: 240,
  };

  return (
    <div style={{ padding: 16, background: '#0B0D0E' }}>
      <div style={{
        display: 'grid',
        gridTemplateColumns: '180px 1fr 200px',
        gap: 12,
        fontSize: 12,
        lineHeight: '20px',
      }}>
        {/* VAULT */}
        <div style={cellStyle}>
          {cellHeader('─ vault ─')}
          <div style={{ padding: '8px 6px', display: 'flex', flexDirection: 'column', gap: 2, flex: 1 }}>
            {files.map((f, i) => (
              <div key={i} style={{
                display: 'flex',
                alignItems: 'center',
                gap: 8,
                padding: '2px 6px',
                paddingLeft: 6 + f.indent * 12,
                background: f.active ? accent : 'transparent',
                color: f.active ? '#0B0D0E' : 'var(--text-muted)',
                fontWeight: f.active ? 500 : 400,
              }}>
                <span style={{ width: 10, display: 'inline-block' }}>{f.active ? '▸' : '○'}</span>
                <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{f.name}</span>
              </div>
            ))}
          </div>
        </div>

        {/* PREVIEW */}
        <div style={cellStyle}>
          {cellHeader('─ preview ─ architecture.md ─')}
          <div style={{ padding: '12px 14px', display: 'flex', flexDirection: 'column', gap: 4, flex: 1 }}>
            {previewLines.map((l, i) => (
              <div key={i} style={{
                color: l.c,
                fontWeight: l.w || 400,
                minHeight: 20,
                whiteSpace: 'pre',
              }}>{l.t || ' '}</div>
            ))}
          </div>
        </div>

        {/* CONTEXT */}
        <div style={cellStyle}>
          {cellHeader('─ context ─')}
          <div style={{ padding: '8px 10px', display: 'flex', flexDirection: 'column', gap: 6, flex: 1 }}>
            {sources.map((s) => (
              <div key={s.i} style={{ display: 'flex', justifyContent: 'space-between', gap: 8 }}>
                <span><span style={{ color: accent }}>[{s.i}]</span> <span style={{ color: 'var(--text-muted)' }}>{s.name}</span></span>
                <span style={{ color: 'var(--accent-warm)' }}>{s.score}</span>
              </div>
            ))}
            <div style={{ marginTop: 8, color: 'var(--text-dim)', fontSize: 11, letterSpacing: '0.08em' }}>─── score ───</div>
            <div style={{
              marginTop: 'auto',
              height: 2,
              background: 'var(--bg-surface)',
              position: 'relative',
            }}>
              <div style={{ position: 'absolute', inset: 0, width: '78%', background: accent }} />
            </div>
          </div>
        </div>
      </div>

      {/* Status bar */}
      <div style={{
        marginTop: 10,
        display: 'flex',
        alignItems: 'center',
        gap: 12,
        fontSize: 11,
        color: 'var(--text-dim)',
        borderTop: '1px solid var(--border-strong)',
        paddingTop: 10,
        flexWrap: 'wrap',
      }}>
        <span style={{
          background: accent,
          color: '#0B0D0E',
          padding: '1px 8px',
          fontWeight: 600,
          letterSpacing: '0.1em',
        }}>NORMAL</span>
        <span><span style={{ color: 'var(--text-muted)' }}>?</span> help</span>
        <span><span style={{ color: 'var(--text-muted)' }}>/</span> search</span>
        <span><span style={{ color: 'var(--text-muted)' }}>:</span> command</span>
        <span><span style={{ color: 'var(--text-muted)' }}>q</span> quit</span>
        <span style={{ marginLeft: 'auto', color: 'var(--text-muted)' }}>~/mald/vault · 1247 notes</span>
        <span className="cli-blink" style={{ color: accent, marginLeft: 4 }} />
      </div>
    </div>
  );
}
function CliBody({ accent }) {
  const lines = [
    { prompt: '$', cmd: 'mald query "what is hnsw?"', out: null },
    { prompt: null, cmd: null, out: '↳ searching local index across 1,247 notes…' },
    { prompt: null, cmd: null, out: '↳ retrieved 3 sources · cosine 0.91 / 0.92 / 0.93' },
    { prompt: null, cmd: null, out: '' },
    { prompt: null, cmd: null, out: 'HNSW (Hierarchical Navigable Small World) is a graph-based' },
    { prompt: null, cmd: null, out: 'approximate nearest neighbor algorithm. MALD uses it to index' },
    { prompt: null, cmd: null, out: 'embeddings of every markdown chunk in your vault. [1][2][3]' },
    { prompt: null, cmd: null, out: '' },
    { prompt: null, cmd: null, out: 'sources:' },
    { prompt: null, cmd: null, out: '  [1] 03_Resources/papers/hnsw.md             0.91', warm: true },
    { prompt: null, cmd: null, out: '  [2] 01_Projects/MALD/architecture.md        0.92', warm: true },
    { prompt: null, cmd: null, out: '  [3] 03_Resources/papers/vector-search.md    0.93', warm: true },
    { prompt: '$', cmd: '_', out: null, blink: true },
  ];
  return (
    <div style={{ padding: 24, fontSize: 13, lineHeight: '22px' }}>
      {lines.map((l, i) => (
        <div key={i} style={{ display: 'flex', gap: 8, color: l.warm ? 'var(--accent-warm)' : 'var(--text-muted)' }}>
          {l.prompt && <span style={{ color: accent }}>{l.prompt}</span>}
          {l.cmd && <span style={{ color: 'var(--text)' }}>
            {l.cmd}
            {l.blink && <span className="cli-blink" style={{ color: accent }}> </span>}
          </span>}
          {l.out !== null && <span style={{ whiteSpace: 'pre' }}>{l.out}</span>}
        </div>
      ))}
    </div>
  );
}

// ---------- Features ----------
function Features({ accent }) {
  const ref = useReveal();
  const items = [
    { Icon: IconZap, title: 'HNSW vector search', body: 'Fast semantic retrieval across your local markdown vault.' },
    { Icon: IconLink, title: 'Markdown wikilinks', body: 'Connect notes using readable links that stay plain on disk.' },
    { Icon: IconGitBranch, title: 'Force-directed graph', body: 'See relationships between projects, notes, ideas, and sources.' },
    { Icon: IconQuote, title: 'Cited RAG chat', body: 'Ask questions and inspect the local files behind each answer.' },
    { Icon: IconLayers, title: 'Multi-space workspaces', body: 'Separate coursework, research, projects, and personal systems.' },
    { Icon: IconActivity, title: 'File watcher daemon', body: 'Keep indexes fresh as your vault changes.' },
  ];
  return (
    <section ref={ref} className="reveal" style={{ maxWidth: 1200, margin: '0 auto', padding: '96px 32px 0' }}>
      <SectionLabel>// features</SectionLabel>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 16 }}>
        {items.map((c, i) => (
          <div key={i} style={{
            background: 'var(--bg-panel)',
            border: '1px solid var(--border)',
            borderRadius: 8,
            padding: 24,
          }}>
            <GlyphHolder accent={accent}><c.Icon size={12} color={accent} /></GlyphHolder>
            <h3 style={{ margin: 0, fontSize: 14, lineHeight: '22px', fontWeight: 500, color: 'var(--text)', marginBottom: 8 }}>{c.title}</h3>
            <p style={{ margin: 0, fontSize: 13, lineHeight: '22px', color: 'var(--text-muted)' }}>{c.body}</p>
          </div>
        ))}
      </div>
    </section>
  );
}

// ---------- Architecture ----------
function Architecture({ accent }) {
  const ref = useReveal();
  return (
    <section ref={ref} className="reveal" style={{ maxWidth: 1200, margin: '0 auto', padding: '96px 32px 0' }}>
      <SectionLabel>// architecture</SectionLabel>
      <div style={{
        background: 'var(--bg-panel)',
        border: '1px solid var(--border)',
        borderRadius: 8,
        padding: 48,
      }}>
        <ArchDiagram accent={accent} />
        <div style={{
          textAlign: 'center',
          marginTop: 40,
          color: 'var(--text-muted)',
          fontSize: 13,
          lineHeight: '22px',
        }}>
          Custom HNSW vector search. SQLite FTS5 fallback. Notify-rs file watcher. Optional Ollama backend.
        </div>
      </div>
    </section>
  );
}

function ArchDiagram({ accent }) {
  // SVG line diagram, fully responsive via viewBox
  const stroke = '#2A2F33';
  const fill = '#171A1D';
  const text = '#E8E2D4';
  const boxW = 140;
  const boxH = 56;
  const r = 8;
  const gap = 80;
  // Layout: 4 boxes in a row, "Markdown files" centered above box #2 (Daemon)
  const totalW = boxW * 4 + gap * 3;
  const startX = (760 - totalW) / 2;
  const rowY = 180;
  const topY = 40;

  const boxes = [
    { x: startX, y: rowY, label: 'CLI' },
    { x: startX + (boxW + gap), y: rowY, label: 'Daemon' },
    { x: startX + (boxW + gap) * 2, y: rowY, label: 'HNSW Index' },
    { x: startX + (boxW + gap) * 3, y: rowY, label: 'Ollama' },
  ];
  // Top box centered above "Daemon"
  const topBox = { x: boxes[1].x, y: topY, label: 'Markdown files' };

  return (
    <div style={{ width: '100%', overflowX: 'auto' }}>
      <svg viewBox={`0 0 760 280`} style={{ width: '100%', height: 'auto', maxWidth: 760, margin: '0 auto', display: 'block' }} preserveAspectRatio="xMidYMid meet">
        {/* Vertical line from Markdown to Daemon */}
        <line x1={topBox.x + boxW / 2} y1={topBox.y + boxH} x2={boxes[1].x + boxW / 2} y2={boxes[1].y} stroke={stroke} strokeWidth="1" />
        {/* Horizontal connectors */}
        {boxes.slice(0, -1).map((b, i) => (
          <line
            key={i}
            x1={b.x + boxW}
            y1={b.y + boxH / 2}
            x2={boxes[i + 1].x}
            y2={boxes[i + 1].y + boxH / 2}
            stroke={stroke}
            strokeWidth="1"
          />
        ))}
        {/* Top box */}
        <g>
          <rect x={topBox.x} y={topBox.y} width={boxW} height={boxH} rx={r} ry={r} fill={fill} stroke={stroke} strokeWidth="1" />
          <text x={topBox.x + boxW / 2} y={topBox.y + boxH / 2 + 4} textAnchor="middle" fill={text} fontFamily="JetBrains Mono, monospace" fontSize="12">{topBox.label}</text>
        </g>
        {/* Row boxes */}
        {boxes.map((b, i) => (
          <g key={i}>
            <rect x={b.x} y={b.y} width={boxW} height={boxH} rx={r} ry={r} fill={fill} stroke={stroke} strokeWidth="1" />
            <text x={b.x + boxW / 2} y={b.y + boxH / 2 + 4} textAnchor="middle" fill={text} fontFamily="JetBrains Mono, monospace" fontSize="12">{b.label}</text>
            {/* Tiny sage corner glyph on Daemon for emphasis */}
            {b.label === 'Daemon' && (
              <circle cx={b.x + 10} cy={b.y + 10} r={2} fill={accent} />
            )}
          </g>
        ))}
      </svg>
    </div>
  );
}

// ---------- Install ----------
function Install({ accent }) {
  const ref = useReveal();
  const [tab, setTab] = React.useState('Windows');
  const [copied, setCopied] = React.useState(false);

  const blocks = {
    Windows: [
      'scoop install mald',
      'scoop bucket add mald https://github.com/NAME0x0/scoop-mald',
    ],
    macOS: [
      'curl -fsSL https://raw.githubusercontent.com/NAME0x0/MALD/main/install.sh | sh',
    ],
    Linux: [
      'curl -fsSL https://raw.githubusercontent.com/NAME0x0/MALD/main/install.sh | sh',
    ],
  };

  const onCopy = () => {
    const text = blocks[tab].map((l) => `> ${l}`).join('\n');
    navigator.clipboard?.writeText(text).catch(() => {});
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <section ref={ref} className="reveal" id="install" style={{ maxWidth: 1200, margin: '0 auto', padding: '96px 32px 0' }}>
      <SectionLabel>// install</SectionLabel>

      <div style={{ display: 'flex', gap: 8, marginBottom: 24 }}>
        {['Windows', 'macOS', 'Linux'].map((t) => {
          const active = tab === t;
          return (
            <button
              key={t}
              onClick={() => setTab(t)}
              style={{
                padding: '6px 16px',
                borderRadius: 999,
                border: active ? `1px solid ${accent}` : '1px solid var(--border)',
                background: active ? 'rgba(143,175,143,0.18)' : 'var(--bg-panel)',
                color: active ? accent : 'var(--text-muted)',
                fontSize: 12,
                fontFamily: 'inherit',
                transition: 'all 150ms ease-out',
              }}
            >
              {t}
            </button>
          );
        })}
      </div>

      <div style={{
        background: 'var(--bg-panel-alt)',
        border: '1px solid var(--border)',
        borderRadius: 8,
        padding: '16px 20px',
        position: 'relative',
      }}>
        <button
          onClick={onCopy}
          aria-label="Copy install commands"
          style={{
            position: 'absolute',
            top: 12,
            right: 12,
            background: 'transparent',
            border: 'none',
            color: copied ? accent : 'var(--text-dim)',
            display: 'inline-flex',
            alignItems: 'center',
            padding: 4,
            transition: 'color 150ms ease-out',
          }}
        >
          {copied ? <IconCheck size={14} /> : <IconCopy size={14} />}
        </button>
        <div style={{ display: 'flex', flexDirection: 'column', fontSize: 13, lineHeight: '22px' }}>
          {blocks[tab].map((line, i) => (
            <div key={i} style={{ paddingBottom: 4, display: 'flex', gap: 8 }}>
              <span style={{ color: accent }}>&gt;</span>
              <span style={{ color: 'var(--text)', wordBreak: 'break-all' }}>{line}</span>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

// ---------- Footer ----------
function Footer({ accent, version }) {
  return (
    <footer style={{
      maxWidth: 1200,
      margin: '96px auto 0',
      padding: '48px 32px',
      borderTop: '1px solid var(--border)',
      display: 'flex',
      justifyContent: 'space-between',
      alignItems: 'flex-start',
      gap: 32,
      flexWrap: 'wrap',
    }}>
      <div>
        <div style={{ color: accent, fontSize: 14, fontWeight: 600, letterSpacing: '0.05em' }}>MALD</div>
        <div style={{ marginTop: 8, color: 'var(--text-dim)', fontSize: 11 }}>
          MIT licensed · {version}
        </div>
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-end', gap: 4 }}>
        {['docs', 'github', 'crates.io', 'scoop'].map((l) => (
          <a key={l} href={`#${l}`} className="footer-link" style={{
            color: 'var(--text-muted)',
            fontSize: 11,
            transition: 'color 150ms ease-out',
          }}
          onMouseEnter={(e) => e.currentTarget.style.color = accent}
          onMouseLeave={(e) => e.currentTarget.style.color = 'var(--text-muted)'}
          >{l}</a>
        ))}
      </div>
    </footer>
  );
}

Object.assign(window, {
  Header, Hero, WhatItIs, ThreeSurfaces, Features, Architecture, Install, Footer,
});
