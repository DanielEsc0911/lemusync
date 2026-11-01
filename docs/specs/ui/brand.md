# Brand

> **Status:** Decided · **Related:** [rules](../../rules.md) (F1, F2, P6, P7), [design-system](design-system.md), [status-and-notifications](status-and-notifications.md), [decisions](../../decisions.md) (D42) · **Code:** — (planned: brand/, xtask/src/brand/, app/lib/design/)

## Purpose
One brand, one source: `brand/` holds the only hand-edited brand values. `cargo xtask brand` regenerates every derived file (Dart tokens, icons, tray pixmaps) from it.

## Direction
90s console hardware, calm and playful, never childish ([design-system §Principles](design-system.md#principles)).
- **Lemur palette:** charcoal for the mask and tail stripes, warm stone-grey fur as console plastic, bone-white rings, amber eyes.
- **Status colors:** the four face-button colors (green, blue, amber, red). A 90s-console lavender is the brand accent; amber is reserved for attention.
- **Surfaces:** plastic panels, a flat tone plus a 1 px two-tone bevel, no shadows. Game cards are cartridge-shaped.
- **Pixel art** is deferred (mascot and empty states only, later). Until then the vector mark fills those places.

## Palette
Primitives are start values; the generator's contrast check is the arbiter, `brand/tokens.toml` is the source.

| Name | Hex | Name | Hex |
|---|---|---|---|
| `charcoal-950` | `#141211` | `amber-300` | `#F6C177` |
| `charcoal-900` | `#1E1B19` | `amber-500` | `#E89B2D` |
| `charcoal-800` | `#2A2623` | `amber-800` | `#7A4A0C` |
| `charcoal-700` | `#3A3531` | `lavender-300` | `#C4B5F0` |
| `stone-600` | `#655E57` | `lavender-500` | `#7E68D6` |
| `stone-550` | `#7D766E` | `lavender-700` | `#5541A8` |
| `stone-500` | `#8C857D` | `green-300` | `#8FD6A0` |
| `stone-400` | `#A9A39B` | `green-500` | `#2F9E55` |
| `stone-300` | `#C6C1B9` | `green-800` | `#1A5E32` |
| `stone-200` | `#DEDAD3` | `blue-300` | `#93BFF2` |
| `stone-100` | `#ECE9E3` | `blue-500` | `#2F74D0` |
| `stone-50` | `#F6F4F0` | `blue-800` | `#1B4682` |
| `bone` | `#FBFAF7` | `red-300` | `#F2A097` |
| | | `red-500` | `#D3402F` |
| | | `red-800` | `#86261B` |

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
| `focus` | Focus ring | Drawn outside the control with a gap |
| `synced`, `onSynced` | All in sync | Synced status and its content |
| `syncing`, `onSyncing` | Work in progress | Scanning and syncing status |
| `attention`, `onAttention` | Needs the user | Locks, conflicts |
| `danger`, `onDanger` | Failed | Errors, destructive actions |
| `offline`, `onOffline` | Not reachable or paused | Offline and paused status |

`brightness` is not a role: it is `dark` for themes whose name starts with `dark`. Themes: `light`, `dark`, `lightHighContrast`, `darkHighContrast`. Flutter reports high contrast only on iOS and Android 14+, so desktop gets the HC themes through an in-app theme override planned for 3d (P5: also config and CLI).

Status to role: synced to `synced`; scanning and syncing to `syncing`; paused and offline to `offline`; attention to `attention`; error to `danger`.

Contrast: text pairs at least 4.5:1, non-text UI pairs at least 3:1 (WCAG 2.2 AA). In the high-contrast themes text is at least 7:1 and non-text at least 4.5:1.

## Type, space, size, shape, opacity, motion
`brand/tokens.toml` is the source for all of them.
- **Type:** Space Grotesk (display, headline), Inter (title, body, label, caption), JetBrains Mono (mono). All variable fonts, bundled.
- **Space:** a 4-pt scale, `xs` to `xxxl`.
- **Size:** touch target (48), icon sizes, bevel, focus ring, ring stroke, mark.
- **Shape:** radius scale `sm`, `md`, `lg`, `pill`.
- **Opacity:** pressed, hover, disabled.
- **Motion:** durations `fast`, `medium`, `slow` and the `standard` and `emphasized` curves; reduce-motion is honoured.

## Mark
Anatomy:
- **Badge:** a rounded console face button with a top bevel line.
- **Tail:** an arc that curls round the head into the sync loop, striped with N rings and ending in an arrowhead. The generator draws it.
- **Head** with ears, **face mask**, **amber eyes**.
- **Status dot** with a symbol (tray only).

Compositions:

| Name | Used for | Notes |
|---|---|---|
| `app` | macOS, Windows, Android, Linux, packaging | Inset badge |
| `appSquare` | iOS | Full-bleed and opaque; the OS rounds the corners |
| `glyph` | In-app | No badge, transparent |
| `tray-synced`, `tray-syncing`, `tray-paused`, `tray-offline`, `tray-attention`, `tray-error` | Tray | Glyph on a small `charcoal-800` disc so it reads on light and dark panels, plus a status dot |

Status dot symbols: check (synced), two arrows (syncing), two bars (paused), a slash (offline, hollow ring), `!` (attention), `x` (error). Ring count, arc, radii and insets are parameters in `marks.toml`. Android adaptive and themed icons are deferred to Phase 4.

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

`cargo xtask brand --check` regenerates in memory and fails on any difference or failed check. It also fails on any hard-coded design literal (color, duration, radius, size, spacing, opacity, font family) in `app/lib` outside the generated tokens. It runs in `just check` and in CI. The Linux tray moves from freedesktop symbolic icons to these pixmaps ([status-and-notifications §Desktop tray](status-and-notifications.md#desktop-tray-and-menu-bar), D42).

To iterate: edit the sources, run `cargo xtask brand`, open the PNGs and the gallery, review the goldens, and commit sources and outputs together. Never hand-edit a generated file.

## Verify
- `V-BRAND-1`: resvg output is byte-identical between the maintainer's Fedora PC and GitHub `ubuntu-latest` (first CI run after Task 4 of [phase-3a-brand](../../progress/plans/phase-3a-brand.md)).
- `V-BRAND-2`: KDE Plasma 6 renders the ARGB32 tray pixmaps sharply at 24, 32 and 48 px (Task 10 Step 4, maintainer); GNOME with AppIndicator later.
- `V-BRAND-3`: Flutter goldens made on Fedora match on `ubuntu-latest` (first CI run after Task 6).
