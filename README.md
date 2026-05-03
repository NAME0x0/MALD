# MALD product site

Static showcase site for [MALD](https://github.com/NAME0x0/MALD).

- **Branch:** `site` (orphan, isolated from `main` — no Rust source on this branch)
- **Stack:** plain HTML + JSX-via-Babel-standalone + custom WebGL shader. No build step.
- **Deploy:** GitHub Pages via `.github/workflows/site.yml` on every push to `site`.
- **URL:** https://NAME0x0.github.io/MALD/

## Local preview

Open `index.html` in any modern browser, or serve the directory:

```bash
python -m http.server 8000
# then visit http://localhost:8000
```

## Files

- `index.html` — entry, sets up CSS tokens, loads scripts via CDN
- `app.jsx` — root React app, composes sections, owns tweak state
- `sections.jsx` — Header / Hero / WhatItIs / ThreeSurfaces / Features / Architecture / Install / Footer
- `shader.jsx` — WebGL hero background (sage scanlines + grain + vignette + drifting glow)
- `icons.jsx` — geometric sage glyphs
- `tweaks-panel.jsx` — live token tweaker (visible on the deployed site for design iteration)
- `favicon.svg` — copied from the main repo
- `.nojekyll` — disables Jekyll processing on GitHub Pages
