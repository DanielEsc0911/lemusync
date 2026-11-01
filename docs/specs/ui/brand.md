# Brand

> **Status:** Decided · **Related:** [rules](../../rules.md) (F1, F2, P6, P7), [design-system](design-system.md), [status-and-notifications](status-and-notifications.md), [decisions](../../decisions.md) (D42) · **Code:** `brand/`, `xtask/src/brand/`, `app/lib/design/`, `app/assets/fonts/`, `app/assets/licenses/`

## Purpose
One brand, one source: `brand/` holds the only hand-edited brand values. `cargo xtask brand` regenerates every derived file (Dart tokens, icons, tray pixmaps) from it.

## Direction
90s console hardware, calm and playful, never childish ([design-system §Principles](design-system.md#principles)).
- **Lemur palette:** charcoal for the mask and tail stripes, warm stone-grey fur as console plastic, bone-white rings, amber eyes.
- **Status colors:** the four face-button colors (green, blue, amber, red). A 90s-console lavender is the brand accent; amber is reserved for attention.
- **Surfaces:** plastic panels, a flat tone plus a 1 px two-tone bevel, no shadows. Game cards are cartridge-shaped.
- **Pixel art** is deferred (mascot and empty states only, later). Until then the vector mark fills those places.

## Palette
The primitives (name to hex) live in `brand/tokens.toml` `[palette]`; that file is the source and the generator's contrast check has the final say.

Roles (21, every theme defines all of them):

| Role | Meaning | Used for |
|---|---|---|
| `surface` | Base panel | Page and card background |
| `surfaceSunken` | Recessed panel | Inputs, wells, list backgrounds |
| `surfaceRaised` | Raised panel | Cards, dialogs, menus |
| `onSurface` | Primary text | Text and icons on surfaces |
| `onSurfaceMuted` | Secondary text | Captions, hints |
| `outline` | Boundary | Input and outlined-control boundaries (at least 3:1) |
| `bevelLight` | Bevel highlight | Decorative bevel, dividers |
| `bevelDark` | Bevel shade | Decorative bevel, dividers |
| `accent` | Brand accent | Primary actions, selection |
| `onAccent` | On accent | Text and icons on `accent` |
| `focus` | Focus ring | Focus ring, drawn outside the control; same primitive as `onSurface` |
| `synced`, `onSynced` | All in sync | Synced status and its content |
| `syncing`, `onSyncing` | Work in progress | Scanning and syncing status |
| `attention`, `onAttention` | Needs the user | Conflicts, foreign leases |
| `danger`, `onDanger` | Failed | Errors, destructive actions |
| `offline`, `onOffline` | Not reachable or paused | Offline and paused status |

`brightness` is not a role: it is `dark` for themes whose name starts with `dark`. Themes: `light`, `dark`, `lightHighContrast`, `darkHighContrast`. Flutter sets `MediaQuery.highContrast` only on iOS and Android API 34+ (`dart:ui` `window.dart`, `accessibilityFeatures.highContrast`, Flutter 3.47.6), so desktop gets the HC themes through an in-app theme override planned for Phase 3d ([roadmap](../../progress/roadmap.md#phase-3-flutter-app-desktop); P5: also config and CLI).

Contrast: text pairs at least 4.5:1, non-text UI pairs at least 3:1 (WCAG 2.2 AA). In the high-contrast themes text is at least 7:1 and non-text at least 4.5:1.

## Type, space, size, shape, opacity, motion
Groups and their use: [design-system §Tokens](design-system.md#tokens). Values: `brand/tokens.toml`. Families: Space Grotesk (display, headline), Inter (title, body, label, caption), JetBrains Mono (mono); all variable fonts, bundled.

## Mark
Anatomy:
- **Badge:** a rounded console face button with a top bevel line.
- **Tail:** an arc that curls around the head into the sync loop, striped with N rings and ending in an arrowhead. The generator draws it.
- **Head** with ears, **face mask**, **amber eyes**.
- **Status dot** with a symbol (tray only).

Compositions:

| Name | Used for | Notes |
|---|---|---|
| `app` | macOS, Windows, Android, Linux, packaging | Inset badge |
| `appSquare` | iOS | Full-bleed and opaque; the OS rounds the corners |
| `glyph` | In-app | No badge, transparent |
| `tray-synced`, `tray-syncing`, `tray-paused`, `tray-offline`, `tray-attention`, `tray-error` | Linux tray (macOS and Windows: [status-and-notifications §Desktop tray](status-and-notifications.md#desktop-tray-and-menu-bar)) | Glyph on a small `charcoal-800` disc so it reads on light and dark panels, plus a status dot |

Status dot symbols: check (synced), two arrows (syncing), two bars (paused), a slash (offline, hollow ring), `!` (attention), `x` (error). The tail is configured by `[tail]` (`cx`, `cy`, `r`, `start_deg`, `sweep_deg`, `width`, `rings`, plus the arrowhead proportions `head_half` and `head_len` as multiples of `width`, defaults 0.9 and 1.4). Any mark may override any of those keys in `[mark.<name>.tail]`; children inherit it through `extends` and override key by key. Tail colors: `ring`, `stripe`, `arrow` (arrowhead, defaults to `stripe`) and optional `tailHalo` (drawn first, `width + 2 * haloWidth` wide; the `haloWidth` param defaults to 0, no halo). Parts can read the derived params `tailCx`, `tailCy`, `tailR`, `tailWidth`, `tailStartX`, `tailStartY`, `tailEndX`, `tailEndY`. Unknown keys, bad tail numbers, bad `symbol`/`hollow` values and colors outside the palette are errors. Parts may only use `href` to `#id` references and SVGs render without reading any file. Ring count, arc, radii and insets are parameters in `marks.toml` (the badge takes `badgeInset`, `badgeSize` and `badgeRadius`; `appSquare` sets them to 0, 256 and 0). Android adaptive and themed icons are deferred to Phase 4.

First drawing (Task 5): a front-facing ring-tailed lemur. Fur head shaped as a rounded triangle with two round ears (pale inner ears), a white face mask, two dark eye patches slanting down-outward, amber eyes with dark pupils and a dark nose. The badge has a 6 px top bevel line. Status dot is centred at (190, 190) with `dotR` 58 (was 56) and 14 px symbols so it reads at 24 px; the tray tail override is unchanged. On the dark tray disc the dark tail bands vanish into the disc, so the tail reads as light dashes; accepted for now.

## Pipeline
Sources: `brand/tokens.toml`, `brand/mark/parts/*.svg` and `brand/mark/marks.toml`. `cargo xtask brand` validates them (every role in every theme, contrast pairs, no unknown placeholder, safe file names) and writes the outputs.

| Output | What | Mark | px | Format |
|---|---|---|---|---|
| `app/lib/design/tokens/*.g.dart` | Dart tokens | — | — | Dart |
| `brand/out/app.svg`, `brand/out/glyph.svg` | Composed marks | `app`, `glyph` | — | SVG |
| `packaging/icons/lemusync.png` | Linux packaging icon | `app` | 256 | PNG |
| Android `mipmap-*/ic_launcher.png` | Launcher icon | `app` | 48 to 192 | PNG |
| iOS `AppIcon.appiconset/` | App icon set | `appSquare` | 20 to 1024 | PNG, opaque |
| macOS `AppIcon.appiconset/` | App icon set | `app` | 16 to 1024 | PNG |
| Windows `app_icon.ico` | App icon | `app` | 16 to 256 | ICO |
| `app/assets/brand/mark.png` (+ `2.0x`, `3.0x`) | In-app mark | `glyph` | 64, 128, 192 | PNG |
| `crates/daemon/assets/tray/<state>-<px>.argb` | Tray pixmaps | `tray-<state>` | 24, 32, 48 | ARGB32 |

Fonts and icons (vendored, not generated): three variable fonts (Space Grotesk, Inter, JetBrains Mono; OFL 1.1) in `app/assets/fonts/`, with source URLs and SHA-256 in `SOURCES.md`, and `lucide_icons_flutter` 4.0.1 (Lucide 1.60.0; ISC, plus MIT for Feather-derived icons). Licenses ship as assets and are registered in `app/lib/design/licenses.dart`. Measured 2026-11-01: the Linux release bundle grew from 23 MB to 28 MB; `lucide.ttf` (Lucide family, used) is tree-shaken once `LsIcons` is referenced; the package's unused `Lucide100`–`Lucide600` weight fonts (≈ 2.9 MB) always ship; a later trim can vendor `lucide.ttf` from lucide 1.60.0 and drop the package. `app/assets/fonts/SOURCES.md` hashes are verified by `cargo xtask brand --check`.

`cargo xtask brand --check` regenerates in memory and fails on any difference or failed check. It also fails on any hard-coded design literal in `app/lib` outside the generated tokens, and on any Dart file it cannot read. It runs in `just check` and in CI.

`cargo xtask brand` writes the seven token files and every mark output above (`xtask/src/brand/{mark,raster,targets}.rs`). `mark.rs` composes a mark: `extends` copies parts, colors and params and the child overrides key by key (a child `parts` replaces the list); a part is an SVG fragment with `{{color.x}}` (palette primitive → `#RRGGBB`) and `{{param.x}}`; the status dot also gets derived `show1`…`show6` and `dotFill`; the tail is drawn from `[tail]`. `raster.rs` renders with resvg and encodes PNG, opaque PNG (8-bit RGB), ICO and ARGB32 (A,R,G,B per pixel, straight alpha, the StatusNotifierItem `IconPixmap` layout). `targets.rs` is the hard-coded table; tests check the iOS and macOS rows against their `Contents.json`. The dev profile builds `tiny-skia`, `resvg` and `usvg` at `opt-level = 3` so `--check` is fast. The parts are the first drawn lemur (Task 5, pending maintainer review). The literal scan (`xtask/src/brand/literals.rs`) reads whole files, so wrapped arguments are caught, and flags four groups: numbers inside `EdgeInsets`, `BorderRadius`, `Radius`, `Size`, `SizedBox`, `BoxConstraints` and `BorderSide` arguments; named design arguments with a number (`fontSize`, `width`, `elevation`, `spacing`, `size` and similar); color literals and `Colors.*` (except `Colors.transparent`), `Curves.*`, `Cubic(`, `Duration(` and alpha/opacity calls; font family strings and `FontWeight.wNNN`. A bare `0` is not a design value. A match whose first line ends in `// brand-literal-ok: <reason>` is exempt.

To iterate: edit the sources, run `cargo xtask brand`, open the PNGs and the gallery, review the goldens, and commit sources and outputs together. Never hand-edit a generated file.

## Verify
- `V-BRAND-1`: resvg output is byte-identical between the maintainer's Fedora PC and GitHub `ubuntu-latest` (first CI run after Task 4 of [phase-3a-brand](../../progress/plans/phase-3a-brand.md)).
- `V-BRAND-2`: KDE Plasma 6 renders the ARGB32 tray pixmaps sharply at 24, 32 and 48 px (Task 10 Step 4 of phase-3a-brand, maintainer); GNOME with AppIndicator later.
- `V-BRAND-3`: Flutter goldens made on Fedora match on `ubuntu-latest` (first CI run after Task 6 of phase-3a-brand).
