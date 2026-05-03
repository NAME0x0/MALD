// MALD site — root app
const { useState, useEffect } = React;

const TWEAK_DEFAULTS = /*EDITMODE-BEGIN*/{
  "accent": "#8FAF8F",
  "shaderIntensity": 1.0,
  "headline": "Terminal-first personal knowledge OS.",
  "subline": "Plain markdown on disk. Local AI. No telemetry, no cloud, no compromise.",
  "version": "v0.5.7"
}/*EDITMODE-END*/;

function App() {
  const [tweaks, setTweak] = useTweaks(TWEAK_DEFAULTS);

  // Inject styles for blink + nav hover
  useEffect(() => {
    const s = document.createElement('style');
    s.textContent = `
      .cli-blink::after { content: '▌'; animation: blink 1s steps(1) infinite; color: ${tweaks.accent}; }
      @keyframes blink { 50% { opacity: 0; } }
      .nav-link { transition: color 150ms ease-out; }
      .nav-link:hover { color: var(--text); }
    `;
    document.head.appendChild(s);
    return () => s.remove();
  }, [tweaks.accent]);

  return (
    <div>
      <Header accent={tweaks.accent} />
      <Hero
        accent={tweaks.accent}
        shaderIntensity={tweaks.shaderIntensity}
        headline={tweaks.headline}
        subline={tweaks.subline}
      />
      <WhatItIs accent={tweaks.accent} />
      <ThreeSurfaces accent={tweaks.accent} />
      <Features accent={tweaks.accent} />
      <Architecture accent={tweaks.accent} />
      <Install accent={tweaks.accent} />
      <Footer accent={tweaks.accent} version={tweaks.version} />

      <TweaksPanel title="Tweaks">
        <TweakSection title="Brand">
          <TweakColor label="Accent (sage)" value={tweaks.accent} onChange={(v) => setTweak('accent', v)} />
          <TweakRadio
            label="Accent preset"
            value={tweaks.accent}
            onChange={(v) => setTweak('accent', v)}
            options={[
              { label: 'Sage', value: '#8FAF8F' },
              { label: 'Amber', value: '#BFA76A' },
              { label: 'Cyan', value: '#7AB0B0' },
              { label: 'Rust', value: '#C67878' },
            ]}
          />
        </TweakSection>
        <TweakSection title="Hero">
          <TweakSlider
            label="Shader intensity"
            min={0} max={2} step={0.05}
            value={tweaks.shaderIntensity}
            onChange={(v) => setTweak('shaderIntensity', v)}
          />
          <TweakText
            label="Headline"
            value={tweaks.headline}
            onChange={(v) => setTweak('headline', v)}
          />
          <TweakText
            label="Sub-line"
            value={tweaks.subline}
            onChange={(v) => setTweak('subline', v)}
          />
        </TweakSection>
        <TweakSection title="Meta">
          <TweakText
            label="Version"
            value={tweaks.version}
            onChange={(v) => setTweak('version', v)}
          />
        </TweakSection>
      </TweaksPanel>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<App />);
