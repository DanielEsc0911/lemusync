//! `brand [--check]`: generate every brand output from `brand/`, or verify they are current.

mod contrast;
mod dart;
mod literals;
mod mark;
mod output;
mod raster;
mod sources;
mod targets;
mod tokens;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use resvg::tiny_skia::Pixmap;
use resvg::usvg;

use tokens::Tokens;

const TOKENS: &str = "brand/tokens.toml";
const MARKS: &str = "brand/mark/marks.toml";
const PARTS: &str = "brand/mark/parts";

/// Composes every mark and encodes every icon and pixmap.
fn mark_outputs(root: &Path, tokens: &Tokens) -> Result<Vec<output::Output>, Vec<String>> {
    let marks = mark::load_marks(&root.join(MARKS))?;
    let parts = root.join(PARTS);
    let mut errors = Vec::new();
    let mut trees: HashMap<&str, usvg::Tree> = HashMap::new();
    let mut svgs: HashMap<&str, String> = HashMap::new();
    let mut pixmaps: HashMap<(&str, u32), Pixmap> = HashMap::new();
    let mut outs = Vec::new();
    let tgts = targets::targets();
    let mut names: Vec<&str> = targets::SVGS.iter().map(|(_, m)| *m).collect();
    names.extend(tgts.iter().map(|t| t.mark));
    for name in names {
        if !svgs.contains_key(name) {
            match mark::compose(name, &marks, tokens, &parts).and_then(|svg| {
                let tree = raster::parse(&svg).map_err(|e| format!("mark `{name}`: {e}"))?;
                Ok((svg, tree))
            }) {
                Ok((svg, tree)) => {
                    svgs.insert(name, svg);
                    trees.insert(name, tree);
                }
                Err(e) => errors.push(e),
            }
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    for (path, name) in targets::SVGS {
        outs.push(output::Output {
            path: PathBuf::from(path),
            bytes: svgs[name].clone().into_bytes(),
            binary: false,
        });
    }
    let mut pixmap = |mark: &'static str, px: u32| -> Result<Pixmap, String> {
        if let Some(p) = pixmaps.get(&(mark, px)) {
            return Ok(p.clone());
        }
        let p = raster::render_tree(&trees[mark], px).map_err(|e| format!("mark `{mark}`: {e}"))?;
        pixmaps.insert((mark, px), p.clone());
        Ok(p)
    };
    for t in &tgts {
        let bytes = match t.format {
            targets::Format::Png => pixmap(t.mark, t.px).and_then(|p| raster::png(&p)),
            targets::Format::PngOpaque => pixmap(t.mark, t.px).and_then(|p| raster::png_opaque(&p)),
            targets::Format::Argb32 => pixmap(t.mark, t.px).map(|p| raster::argb32(&p)),
            targets::Format::Ico(sizes) => sizes
                .iter()
                .map(|&s| pixmap(targets::mark_for_size(t.mark, s), s))
                .collect::<Result<Vec<_>, _>>()
                .and_then(|ps| raster::ico(&ps)),
        };
        match bytes {
            Ok(bytes) => outs.push(output::Output {
                path: PathBuf::from(&t.path),
                bytes,
                binary: true,
            }),
            Err(e) => errors.push(format!("{}: {e}", t.path)),
        }
    }
    if errors.is_empty() {
        Ok(outs)
    } else {
        Err(errors)
    }
}

pub fn run(root: &Path, check: bool) -> Result<(), Vec<String>> {
    let tokens = Tokens::load(&root.join(TOKENS))?;
    let mut outs = dart::emit(&tokens);
    outs.extend(mark_outputs(root, &tokens)?);
    if !check {
        output::write_all(root, &outs)?;
        println!("brand: wrote {} files", outs.len());
        return Ok(());
    }
    if !output::BYTE_EXACT && outs.iter().any(|o| o.binary) {
        eprintln!("warning: binary outputs are only checked for existence on this host");
    }
    let mut errors: Vec<String> = output::check_all(root, &outs)
        .into_iter()
        .map(|p| format!("{}: out of date, run `cargo xtask brand`", p.display()))
        .collect();
    errors.extend(sources::check(root));
    let scan = literals::scan(root);
    errors.extend(scan.errors);
    errors.extend(scan.hits.into_iter().map(|hit| {
        format!("{hit}: hard-coded design literal (use a token, or end the line with `// brand-literal-ok: <reason>`)")
    }));
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
