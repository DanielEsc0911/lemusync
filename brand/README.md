# brand

The only hand-edited brand sources. Spec: [docs/specs/ui/brand.md](../docs/specs/ui/brand.md).

- `tokens.toml`: palette, colors per theme, contrast pairs, type, space, size, shape, opacity, motion.

To change the brand: edit a source, run `just brand`, review the result, and commit sources and outputs together. Never edit a generated file; `cargo xtask brand --check` (part of `just check` and CI) fails when one is stale.

No color, duration, curve, radius, size, spacing, opacity, font-family or font-weight literal may appear in `app/lib/` outside `app/lib/design/tokens/`. The check reads whole files (wrapped arguments included), allows a bare `0`, and names the offending lines. A true one-off ends with `// brand-literal-ok: <reason>` on the line where the literal starts.

Outputs today: `app/lib/design/tokens/{colors,type,space,size,shape,opacity,motion}.g.dart`. The mark and icon files come later.
