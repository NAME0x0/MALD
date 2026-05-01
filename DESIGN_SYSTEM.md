# MALD Design System

**Latest Update:** 2026-05-01
**Theme:** Catppuccin Mocha (Modified) + Catppuccin Latte (light variant)

This system defines the architectural and visual rules for MALD. It is the single source of truth for all UI implementation. No hardcoded values allowed in the codebase.

---

## 1. Principles
- **Inevitability:** Every screen should feel like no other design was strictly possible.
- **Silence:** The interface recedes; the content projects.
- **Precision:** 1px off is wrong. Alignment is structural.
- **Terminal-first identity (Phase 12):** Green-on-black, mono everywhere. Default font is `iced::Font::MONOSPACE`. The brand accent is `colors::ACCENT` (alias of `GREEN`); BLUE is no longer the primary.

---

## 2. Color Palette (Catppuccin Mocha)

| Token | Hex | Role |
|-------|-----|------|
| `CRUST` | `#11111b` | Activity bar, Status bar, deepest backgrounds |
| `MANTLE` | `#181825` | Sidebar, Panels, Terminal |
| `BASE` | `#1e1e2e` | Editor background, Main canvas |
| `SURFACE0` | `#313244` | Borders, Inactive tabs, Dividers |
| `SURFACE1` | `#45475a` | Hover states, Selection backgrounds |
| `SURFACE2` | `#585b70` | Comments, Line numbers, Disabled text |
| `TEXT` | `#cdd6f4` | Primary text |
| `SUBTEXT1` | `#bac2de` | Secondary text |
| `ACCENT` | `#a6e3a1` | **Brand accent** (alias of GREEN) — primary, focus rings, active tabs, daemon-OK, score chips |
| `LAVENDER` | `#b4befe` | Wikilinks |
| `TEAL` | `#94e2d5` | Headings, secondary highlights |
| `GREEN` | `#a6e3a1` | Source colour for ACCENT — also success/done |
| `RED` | `#f38ba8` | Errors, Destructive actions, daemon-stopped |
| `YELLOW` | `#f9e2af` | Warnings, Tags, modified-tab dot |
| `BLUE` | `#89b4fa` | Reserved — used only for legacy contexts; do not reach for as primary |

---

## 3. Typography Scale

Typeface: **monospace by default** (Phase 12 pivot — set via `iced::Font::MONOSPACE` on the application builder). The renderer falls back through the platform's mono fonts (Cascadia Mono / Menlo / DejaVu Sans Mono). Long prose body sections may opt back into a sans face only when a code-driven readability test fails.

| Token | Size | Weight | Usage |
|-------|------|--------|-------|
| `DISPLAY` | 32px | Bold | App Title / Hero |
| `H1` | 24px | SemiBold | Page Headings |
| `H2` | 20px | Medium | Section Headers |
| `H3` | 16px | Medium | Card Titles, Subsections |
| `BODY` | 14px | Regular | Main UI Text, Code |
| `UI` | 12px | Regular | Sidebar items, Metadata |
| `CAPTION` | 10px | Medium | Badges, Tiny Labels |

---

## 4. Spacing System (4px Grid)

whitespace is active structural material.

| Token | Value | usage |
|-------|-------|-------|
| `XS` | 4px | Icon-to-text gap, compact grouping |
| `SM` | 8px | Standard component padding, list items |
| `MD` | 12px | Card padding, Section separation |
| `LG` | 16px | Container padding, Major component gap |
| `XL` | 24px | Page margins, Modal padding |
| `XXL` | 32px | Hero sections |

**Rule:** Margin/Padding must always be a multiple of 4.

---

## 5. Layout & Geometry

| Token | Value | Notes |
|-------|-------|-------|
| `RADIUS_SM` | 4px | Buttons, Inputs, Badges |
| `RADIUS_MD` | 8px | Cards, Modals, Popovers |
| `RADIUS_LG` | 16px | Large Containers |
| `ACTIVITY_BAR` | 52px | Fixed width |
| `SIDEBAR` | 250px | Default width, pure resizing |
| `STATUS_BAR` | 24px | Fixed height |

---

## 6. Motion & Animation

### Core Timing Tokens

| Token | Duration | Curve | Usage |
|-------|----------|-------|-------|
| `HOVER` | 150ms | EaseOut | Button hovers, List item highlights |
| `PRESS` | 100ms | Linear | Active states |
| `COLLAPSE` | 200ms | EaseInOutQuad | Panel toggles (legacy, prefer velocity-aware) |
| `TAB_CLOSE` | 150ms | EaseOut | Tab creation/closing |

### Phase 1 Motion Tokens (2026-02-05)

| Token | Value | Usage |
|-------|-------|-------|
| `TOAST_ENTER` | 200ms | Toast slide-up + fade-in (ease_out_quint) |
| `TOAST_EXIT` | 150ms | Toast fade-out + slide-down (ease_in_quad) |
| `TOAST_AUTO_DISMISS` | 4000ms | Display time before auto-exit |
| `MODAL_FADE_IN` | 150ms | Modal overlay + content fade in |
| `MODAL_FADE_OUT` | 100ms | Modal exit (faster than enter) |
| `PANEL_PIXELS_PER_SEC` | 1200.0 | Velocity for panel animations |

### Velocity-Aware Animations

Panel animations (sidebar, terminal, feature panel) use velocity-based duration:
- **Formula:** `duration = distance / PANEL_PIXELS_PER_SEC`
- **Clamped:** 120ms minimum (prevents jank), 350ms maximum (prevents sluggishness)
- **Easing:** `ease_out_quint` (snappy start, gentle settle)

### Easing Functions

| Function | Formula | Character | Usage |
|----------|---------|-----------|-------|
| `ease_out_quad` | 1-(1-t)² | Smooth decel | General reveals |
| `ease_in_quad` | t² | Smooth accel | Exits, dismissals |
| `ease_in_out_quad` | Symmetric | Balanced | Legacy panels |
| `ease_out_quint` | 1-(1-t)⁵ | Snappy settle | Velocity-aware panels, toast enter |
| `ease_out_back` | Overshoot 1.7 | Springy | Button press micro-feedback |

**Philosophy:** Physics > Easing > Linear. Things have weight.
- Enter slower than exit (users wait for entrance, exits shouldn't block)
- Duration scales with distance (fixed timing feels wrong)
- Asymmetric timing creates intentionality

---

## 7. Focus States

| Token | Value | Usage |
|-------|-------|-------|
| `FOCUS_RING_WIDTH` | 2px | Focus indicator width |
| `FOCUS_RING_OFFSET` | 2px | Space between element and ring |
| `FOCUS_RING_COLOR` | BLUE | Focus indicator color |

**Rule:** All interactive elements must have visible focus states for keyboard navigation.

---

## 8. Button Padding Classes

| Token | Value | Usage |
|-------|-------|-------|
| `BTN_PADDING_SM` | [4px, 8px] | Icon buttons, close buttons |
| `BTN_PADDING_MD` | [8px, 12px] | Tab buttons, list items |
| `BTN_PADDING_LG` | [8px, 16px] | Primary/Secondary buttons |

**Rule:** Button padding must always use these classes. No hardcoded values.

---

## 9. Empty State Styling

| Token | Hex | Role |
|-------|-----|------|
| `EMPTY_ICON` | SUBTEXT0 | Empty state icons (visible but muted) |
| `EMPTY_BG` | SURFACE0 | Empty state icon background circle |

**Philosophy:** Empty states are designed states, not absences.

---

## 10. Toast Notifications

| Token | Value | Usage |
|-------|-------|-------|
| `TOAST_WIDTH` | 320px | Fixed toast width |
| `TOAST_SHADOW` | 0 2px 8px rgba(0,0,0,0.25) | Toast elevation |
| `TOAST_RADIUS` | 6px | Toast border radius |
| `TOAST_INFO` | BLUE | Info toast accent |
| `TOAST_SUCCESS` | GREEN | Success toast accent |
| `TOAST_WARNING` | YELLOW | Warning toast accent |
| `TOAST_ERROR` | RED | Error toast accent |

---

## 11. View Header Standards

| Token | Value | Usage |
|-------|-------|-------|
| `VIEW_HEADER_PADDING` | [SM, LG] | Consistent view header padding (8px vertical, 16px horizontal) |

**Rule:** All view headers must use this padding for consistent horizontal breathing room.

---

## 12. Empty State Circle

| Token | Value | Usage |
|-------|-------|-------|
| `EMPTY_STATE_CIRCLE` | 64px | Fixed size for empty state icon backgrounds |
| `EMPTY_STATE_RADIUS` | 32px | Half of circle size for perfect circle |

**Rule:** Empty state icons are centered in a 64x64 circular container with SURFACE0 background.

---

## 13. Scrollbar Visibility

| Token | Value | Usage |
|-------|-------|-------|
| `SCROLLBAR_RAIL_OPACITY` | 0.3 | Subtle visibility for scroll rails |

**Philosophy:** Users should know content is scrollable before interacting. Invisible scrollbars fail the "no thinking required" test.

---

## 14. Icon Size Reference Rule

**Rule:** All icon sizes must reference `icon_size::*` tokens, even when the desired size matches a token value exactly. No hardcoded icon sizes.

| Token | Value | Usage |
|-------|-------|-------|
| `PRIMARY` | 16px | Activity bar, primary actions |
| `SECONDARY` | 14px | Close buttons, chevrons, search icons |
| `INLINE` | 12px | Inline indicators |

---

## 15. Additional Layout Tokens

| Token | Value | Usage |
|-------|-------|-------|
| `PANEL_HEADER_HEIGHT` | 32px | Feature panel, sidebar section headers |
| `SEARCH_INPUT_WIDTH` | 400px | Top search bar input width |
| `CLOSE_BUTTON_SIZE` | 16px | Tab close buttons, panel close buttons |

**Rule:** Component-specific dimensions must be defined in the layout module, not as local constants.
