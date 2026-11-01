# brand

The only hand-edited brand sources. Spec: [docs/specs/ui/brand.md](../docs/specs/ui/brand.md).

- `tokens.toml`: palette, colors per theme, contrast pairs, type, space, size, shape, opacity, motion.
- `mark/marks.toml` and `mark/parts/*.svg`: the lemur mark. Parts are SVG fragments (256×256 space) with `{{color.x}}` and `{{param.x}}` placeholders; `marks.toml` composes them (`extends`, colors by palette name, params) and sets the built-in tail. Part names match `^[A-Za-z0-9@._-]+$`.

To change the brand: edit a source, run `just brand`, review the result, and commit sources and outputs together. Never edit a generated file; `cargo xtask brand --check` (part of `just check` and CI) fails when one is stale.

No color, duration, curve, radius, size, spacing, opacity, font-family or font-weight literal may appear in `app/lib/` outside `app/lib/design/tokens/`. The check reads whole files (wrapped arguments included), allows a bare `0`, and names the offending lines. A true one-off ends with `// brand-literal-ok: <reason>` on the line where the literal starts.

Outputs:
- `app/lib/design/tokens/{colors,type,space,size,shape,opacity,motion}.g.dart`
- `brand/out/{app,glyph}.svg`
- `packaging/icons/lemusync.png`
- Android `app/android/app/src/main/res/mipmap-*/ic_launcher.png`
- iOS and macOS `AppIcon.appiconset/*.png` (the files named in each `Contents.json`)
- `app/windows/runner/resources/app_icon.ico`
- `app/assets/brand/{,2.0x/,3.0x/}mark.png`
- `crates/daemon/assets/tray/<state>-<24|32|48>.argb` (raw ARGB32, big-endian, `4·px·px` bytes)

Binary outputs are byte-compared on Linux x86_64 only; elsewhere `--check` only requires them to exist.
