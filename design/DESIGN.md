---
name: Penumbra
description: Penumbra on Axiom. Full migration to the Axiom style (Nov 12 2026); one orange, ember #da5c2c, for the Sun, the wordmark and UI accents (team pick, Nov 12). Base text is design/brand/axiom-source.md, tokens design/brand/axiom-tokens.json.
colors:
  void: "#000000"
  carbon: "#111111"
  graphite: "#191919"
  iron: "#202020"
  slate: "#3a3a3a"
  pewter: "#505050"
  steel: "#606060"
  ash: "#7e7e7e"
  fog: "#b4b4b4"
  chalk: "#d9d9d9"
  paper: "#eeeeee"
  ember: "#da5c2c"
  sun: "#da5c2c"
  earth: "#0032A0"
  earth-line: "#4f7fd9"
typography:
  mono:
    fontFamily: "IBM Plex Mono, ui-monospace, monospace"
    fontWeight: 400
  mono-bold:
    fontFamily: "IBM Plex Mono, ui-monospace, monospace"
    fontWeight: 700
  fine:
    fontFamily: "Inter, system-ui, sans-serif"
    fontWeight: 400
  wordmark:
    fontFamily: "Archivo, sans-serif"
    fontWeight: 800
    fontStyle: italic
    fontVariation: "'wdth' 112"
rounded:
  sm: "2px"
  full: "9999px"
spacing:
  unit: "8px"
---

# Penumbra on Axiom

> Terminal window at midnight: flat black canvas, monospaced text, and one low Sun on the horizon.

**Theme:** dark only.

This file is the source of truth. `tokens.css` mirrors it; change a value here first.

- **Sections 1–9** are Axiom, taken from `design/brand/axiom-source.md` with three edits, each marked **[Penumbra]**:
  - IBM Plex Mono replaces BerkeleyMono.
  - NASA blue replaces Axiom's `#2a7fff`.
  - One rule for ember, which the source contradicts itself on: it is now Penumbra's only warm colour, and buttons carry dark text.
- **Section 10** covers what Axiom never had: the Sun, the wordmark, charts, FAKE labels and slides.

Axiom is a terminal-grade dark interface:
- **Surfaces:** every glyph is monospaced and every surface is near-black. Information is layered through small shifts in dark neutrals (`#000000` canvas → `#191919` cards → `#202020` borders), so the UI reads like a well-formatted log file.
- **What it leaves out:** no gradients, almost no shadows, no rounded softness.
- **Type:** one monospace carries the whole voice, from headlines to labels.
- **Colour:** one orange, Axiom's ember, appears only as a tool, never as decoration. It marks the Sun and sunlight, and a few interface accents.

## 1. Colours

| Name | Value | Token | Role |
|------|-------|-------|------|
| Void | `#000000` | `--color-void` | Page background, nav fill, terminal surface, slide ground |
| Carbon | `#111111` | `--color-carbon` | Primary surface, card base, chart panels |
| Graphite | `#191919` | `--color-graphite` | Elevated card background, callout fill |
| Iron | `#202020` | `--color-iron` | Hairline borders, dividers, subtle separation |
| Slate | `#3a3a3a` | `--color-slate` | Muted borders, icon strokes, secondary lines, `>` texture, terrain in shadow |
| Pewter | `#505050` | `--color-pewter` | Subtle dividers, decorative rules |
| Steel | `#606060` | `--color-steel` | Tertiary text, inactive labels (3.3:1 on void: 18px and up only) |
| Ash | `#7e7e7e` | `--color-ash` | Muted helper text, small annotations, axis labels, lit terrain |
| Fog | `#b4b4b4` | `--color-fog` | Secondary text, label text, muted body |
| Chalk | `#d9d9d9` | `--color-chalk` | Light borders, stroke detail on icons |
| Paper | `#eeeeee` | `--color-paper` | Primary text, headings, button text, lethal ✕ |
| Ember | `#da5c2c` | `--color-ember` | The one orange: Sun, wordmark Sun, and the UI accents in the ember rule below |
| **[Penumbra] Sun** | `#da5c2c` | `--color-sun` | Alias of ember for sunlight: Sun track, Sun in the wordmark, sunlit hours |
| **[Penumbra] Earth** | `#0032A0` | `--color-earth` | NASA blue. Earth contact as a **filled** area or block |
| **[Penumbra] Earth line** | `#4f7fd9` | `--color-earth-line` | Same hue as Earth, lightened. Earth track lines and Earth text on dark grounds |

**[Penumbra] Ember rule.** The source both forbids ember as the CTA colour (its colour table) and requires it (its Do list). Penumbra settles it: ember is the only orange.
- **Use ember for:**
  - the Sun: Sun track, sunlit hours, the Sun in the wordmark (as `--color-sun`)
  - the primary button fill
  - log-bar patterns
  - the 2px left border on callout cards
- **Never use ember for:**
  - text
  - chart data lines

**[Penumbra] Blue rule.** NASA blue `#0032A0` measures about 2:1 on void, which is too dark for a thin line or text. Use it for fills, and `#4f7fd9` (5.4:1 on void) for lines and text.

## 2. Typography

### IBM Plex Mono · universal typeface · `--font-mono`

**[Penumbra]** Axiom uses BerkeleyMono, a paid font. IBM Plex Mono is one of its listed substitutes, free (OFL), and it has tabular figures.

It is used for everything: headlines, body, nav, buttons, code, table cells, chart labels.
- **Weights:** 400 carries prose and UI labels; 700 is reserved for emphasis and primary buttons. These are the only weights.
- **Sizes:** 12, 13, 14, 16, 18, 20, 24 and 32px.
- **Line height:** 1.0–1.71.
- **Letter spacing:** normal.
- **No proportional fallback** for content.

### Inter · fine print only · `--font-fine`

Small icon labels, fine print and tertiary annotations, where proportional letterforms lighten density below the mono baseline.
- **Weight:** 400
- **Sizes:** 12, 14 and 16px
- **Line height:** 1.33–1.50

Never use it for headlines, nav or buttons.

### Type scale (web)

| Role | Family | Weight | Size | Line height | Token |
|------|--------|--------|------|-------------|-------|
| caption | Mono | 400 | 12px | 1.5 | `--text-caption` |
| body | Mono | 400 | 14px | 1.71 | `--text-body` |
| heading-sm | Mono | 400 | 18px | 1.56 | `--text-heading-sm` |
| heading | Mono | 400 | 20px | 1.4 | `--text-heading` |
| heading-lg | Mono | 400 | 24px | 1.33 | `--text-heading-lg` |
| display | Mono | 400 | 32px | 1.25 | `--text-display` |

The full extended steps (xs … 3xl) are in `design/brand/axiom-tokens.json` and `tokens.css`. The slide scale is in section 10.

## 3. Spacing and shape

- **Spacing:**
  - Base unit 8px, comfortable density.
  - Scale: 8, 16, 24, 32, 40, 48, 64, 80, 96 (`--spacing-*`).
- **Radius:**
  - Cards, inputs, buttons and panels: **2px**.
  - Tags, pills and icon badges under 32px: 9999px.
  - Nothing else is rounded above 2px.
- **Shadow:** only `--shadow-subtle` (`rgba(0,0,0,0.05) 0 1px 2px`), on buttons and small interactive surfaces. Elevation comes from surface steps, never shadows.
- **Layout:**
  - Page max-width 1200px.
  - Section gap 40px (40–64px between sections).
  - Card padding 32px.
  - Element gap 16px.

### Surfaces

| Level | Name | Value | Purpose |
|-------|------|-------|---------|
| 0 | Void | `#000000` | Page base, nav, terminal canvas |
| 1 | Carbon | `#111111` | Primary content surface, chart panels |
| 2 | Graphite | `#191919` | Elevated cards, inset blocks |
| 3 | Iron | `#202020` | Borders, dividers, inner separation |

## 4. Components (Axiom)

- **Primary button:**
  - Ember `#da5c2c` fill, **void text** (5.5:1), Mono 16px weight 700, 2px radius, padding 10px 16px.
  - **[Penumbra]** Axiom's paper text on ember is 3.3:1 and fails AA at 16px, so the label is void.
  - A `→` follows the label.
  - No shadow and no hover lift; hover changes colour only.
- **Ghost button:** transparent, 1px slate border, paper text, Mono 16px weight 400, 2px radius, 10px 16px padding, with `→`.
- **Top bar:**
  - Void background, full width, 16px horizontal padding, about 56px tall.
  - Wordmark on the left; links in Mono 14px paper, with optional dropdown chevrons.
  - Optional hairline iron bottom border.
- **Announcement strip:** full-width carbon strip, Mono 12px paper text, with a close × on the right. Not colourful.
- **Callout card** (Axiom's case-study card):
  - Graphite background, 2px radius, 32px padding.
  - A 2px ember left border runs the full height: the only colour in the card.
  - Headline Mono 18px paper, body 14px.
  - Nested carbon sub-card for quotes.
  - An underlined `link →` at the bottom.
- **Screenshot panel:**
  - Carbon frame, 2px radius, iron inner dividers.
  - Tab bar in Mono 12px fog, with the active tab underlined in paper.
  - Results tables in Mono 12px with slate row dividers.
- **Arrow field:** repeating `>` glyphs in Mono 12px slate, set on a diagonal, filling negative space beside headline copy. It signals streaming data.
- **Tag:** Mono 12px ash uppercase (Axiom uses steel, but that is 3.0:1 on carbon; steel only at 18px+), no fill, no border, set above a headline as metadata.
- **Pill badge:** 9999px radius, about 24px diameter, carbon or iron fill, white glyph.
- **Arrow link:** Mono 14px paper, underlined, followed by `→`.
- **Log-bar histogram:**
  - Tight rows of bars 4–8px tall, with 2–4px gaps, in ember or **[Penumbra]** earth.
  - No axis labels in a marketing context; the pattern is the picture.

## 5. Do

- Use the mono for all UI text, including nav, buttons and labels.
- Layer surfaces void → carbon → graphite → iron; never use box-shadow for elevation.
- Set radius to 2px on containers, cards and buttons. Keep 9999px for badges and pills under 32px.
- Put the 2px ember left border on editorial and callout cards as a category mark.
- Use `→` after action labels, for the CLI cadence.
- Anchor the page on void and let paper text carry hierarchy through weight (400 / 700) and size.
- Open sections with a terminal prompt label: `~/section`.

## 6. Don't

- Don't add gradients, shadows beyond the single hairline, or coloured backgrounds. The system is flat.
- Don't use proportional fonts for headlines, nav or buttons.
- Don't scatter ember: use it only where the ember rule allows.
- Don't round corners above 2px on cards, panels or buttons.
- Don't add light-theme sections. The product is dark only.
- Don't use coloured badges for status. Show state with mono symbols (✓ ✕ ▲) and borders.
- Don't use photography or lifestyle imagery. Visuals are product UI, terrain, log patterns and `>` textures.

## 7. Imagery

Visuals are the product's own UI: horizon panoramas, timelines, monospaced tables. Decoration is limited to two things:
- the `>` field in slate
- log-bar terrain

Logos, where they appear, are grayscale (paper or fog) on carbon. No photography and no 3D renders: the product screenshot is the hero.

## 8. Layout

Full-bleed void canvas, content in a centred column up to 1200px.
- **Hero:**
  - A left-aligned block: the `~/` prompt above a two-line mono headline.
  - The `>` texture on the right, and a full-width product panel below.
- **Section rhythm:** steady, 40–64px apart.
- **Nav:** a plain top bar, no mega-menu.

## 9. Accessibility (measured on void `#000000`)

| Foreground | Ratio | Use |
|---|---|---|
| Paper `#eeeeee` | 18.1:1 | any text |
| Fog `#b4b4b4` | 10.1:1 | any text |
| Ash `#7e7e7e` | 5.2:1 | small text OK |
| Earth line `#4f7fd9` | 5.4:1 | lines, labels |
| Ember / Sun `#da5c2c` | 5.5:1 | Sun line and label, fills, bars, borders |
| Void on ember | 5.5:1 | primary button text |
| Steel `#606060` | 3.3:1 | 18px+ or non-text only |
| Earth `#0032A0` | 2.0:1 | fills only, never lines or text |

Meaning never rests on colour alone: Sun and Earth tracks also differ in lightness and carry a text label, and lethal and pass/fail use symbols.

## 10. [Penumbra] Additions, not in Axiom

### Wordmark: kept from Low Sun
- **The one exception to "mono everywhere".** The PENUMBRA wordmark is unchanged: Archivo italic 800, `wdth` 112, letters in paper `#eeeeee`.
- **The mark:** the half-risen starburst Sun on its hairline horizon, in **ember `#da5c2c`** (was amber `#f4a740` until Nov 12).
- **Source:** `design/brand/index.html` (set A, mark 1 · Grazing Sun) and `design/brand/assets/sun-mark.svg`.
- **Build rule:** measure on the loaded font, never stretch with `textLength`.
- The small mark (`sun-mark-16.svg`) is the favicon.

### Sun means Sun
- `--color-sun` (ember) is the only colour for sunlight: the Sun track, sunlit hours, the Sun in the mark.
- On a chart that shows the Sun, nothing else is ember: log bars and accents there go slate or Earth blue, so orange always means Sun.

### Data encoding
| Meaning | Encoding |
|---|---|
| Sun track | Sun (ember) `#da5c2c` line, 2.5px, label "Sun" |
| Earth track | Earth-line `#4f7fd9`, 2px dashed, label "Earth"; Earth `#0032A0` for filled bands |
| Terrain / horizon | Log-bar ridge: ash where lit, slate in shadow |
| First lethal shadow | **✕** in paper, weight 700, plus the words "first lethal shadow". No red. |
| PASS / MISS | `✓ PASS` in fog / `✕ MISS` in paper 700 |
| Axes, ticks | Ash text, iron gridlines |

### FAKE label
- A pill: 9999px radius, 1px slate border, Mono 12px ash uppercase, text "FAKE DATA" or "PLACEHOLDER".
- Put it on every chart, table or number that is not real, in the panel's top right.

### Charts
- Panels are carbon with a 2px radius and iron dividers.
- One idea per chart. Direct labels at the line end, no legend boxes.
- Numbers in tabular mono.

### Slides (1920×1080)
- **Background:** void.
- **Margins:** 120px left and right.
- **Section label:** `~/section` top left.
- **Footer:** wordmark-free, with the page counter `0n / 07` in ash and progress bars (ember for the current slide, slate for the rest).
- **Type scale** (Axiom's web scale ×2, mono throughout):

| Role | Size | Weight |
|---|---|---|
| display | 64px | 700 |
| title | 48px | 400 |
| body | 28px | 400 |
| caption / label | 22px | 400 |
| data | 28px | 400, tabular |

### Motion
- Axiom is nearly static. Motion is limited to a blinking cursor and data drawing in once (≤ 400ms, ease-out).
- Respect `prefers-reduced-motion`.
