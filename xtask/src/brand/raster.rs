//! SVG → pixmap, and the PNG, opaque PNG, ICO and ARGB32 encoders.

use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg;

/// Parses `svg`. Mark SVGs are data: no `<image>` may pull in a file (data URLs stay allowed).
pub fn parse(svg: &str) -> Result<usvg::Tree, String> {
    let opt = usvg::Options {
        image_href_resolver: usvg::ImageHrefResolver {
            resolve_data: usvg::ImageHrefResolver::default_data_resolver(),
            resolve_string: Box::new(|_, _| None),
        },
        ..usvg::Options::default()
    };
    usvg::Tree::from_str(svg, &opt).map_err(|e| format!("svg: {e}"))
}

/// Renders a parsed tree square at `px` × `px`.
pub fn render_tree(tree: &usvg::Tree, px: u32) -> Result<Pixmap, String> {
    let mut pm = Pixmap::new(px, px).ok_or_else(|| format!("cannot allocate a {px}px pixmap"))?;
    let side = u16::try_from(px).map_err(|_| format!("{px}px is too large"))?;
    let k = f32::from(side) / tree.size().width();
    resvg::render(tree, Transform::from_scale(k, k), &mut pm.as_mut());
    Ok(pm)
}

/// Renders `svg` square at `px` × `px`.
#[cfg(test)]
pub fn render(svg: &str, px: u32) -> Result<Pixmap, String> {
    render_tree(&parse(svg)?, px)
}

/// Straight-alpha RGBA bytes.
fn rgba(p: &Pixmap) -> Vec<u8> {
    p.pixels()
        .iter()
        .flat_map(|px| {
            let c = px.demultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        })
        .collect()
}

pub fn png(p: &Pixmap) -> Result<Vec<u8>, String> {
    p.encode_png().map_err(|e| format!("png: {e}"))
}

/// 8-bit RGB PNG without an alpha channel (iOS rejects icons with one).
pub fn png_opaque(p: &Pixmap) -> Result<Vec<u8>, String> {
    if p.pixels().iter().any(|px| px.alpha() != 255) {
        return Err("png_opaque: mark has transparent pixels".into());
    }
    let rgb: Vec<u8> = p
        .pixels()
        .iter()
        .flat_map(|px| {
            let c = px.demultiply();
            [c.red(), c.green(), c.blue()]
        })
        .collect();
    let mut out = Vec::new();
    let mut enc = png::Encoder::new(&mut out, p.width(), p.height());
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()
        .and_then(|mut w| w.write_image_data(&rgb))
        .map_err(|e| format!("png: {e}"))?;
    Ok(out)
}

/// One ICO with every pixmap as an image.
pub fn ico(ps: &[Pixmap]) -> Result<Vec<u8>, String> {
    let mut dir = ico::IconDir::new(ico::ResourceType::Icon);
    for p in ps {
        let img = ico::IconImage::from_rgba_data(p.width(), p.height(), rgba(p));
        let entry = ico::IconDirEntry::encode(&img).map_err(|e| format!("ico: {e}"))?;
        dir.add_entry(entry);
    }
    let mut out = Vec::new();
    dir.write(&mut out).map_err(|e| format!("ico: {e}"))?;
    Ok(out)
}

/// `StatusNotifierItem` `IconPixmap` layout: per pixel A, R, G, B, straight alpha.
pub fn argb32(p: &Pixmap) -> Vec<u8> {
    p.pixels()
        .iter()
        .flat_map(|px| {
            let c = px.demultiply();
            [c.alpha(), c.red(), c.green(), c.blue()]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    const RED: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 4 4" width="4" height="4"><rect width="4" height="4" fill="#FF0000"/></svg>"##;
    const HALF: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 4 4" width="4" height="4"><rect width="4" height="4" fill="#FF0000" fill-opacity="0.5"/></svg>"##;

    #[test]
    fn renders_red() {
        let p = render(RED, 4).unwrap();
        let c = p.pixel(1, 1).unwrap().demultiply();
        assert_eq!((c.red(), c.green(), c.blue(), c.alpha()), (255, 0, 0, 255));
        assert_eq!(render(RED, 8).unwrap().width(), 8);
    }

    #[test]
    fn argb32_is_big_endian_and_demultiplied() {
        assert_eq!(
            &argb32(&render(RED, 4).unwrap())[..4],
            &[0xFF, 0xFF, 0x00, 0x00]
        );
        let h = argb32(&render(HALF, 4).unwrap());
        assert!((i32::from(h[0]) - 0x80).abs() <= 1, "alpha {}", h[0]);
        assert_eq!(h[1], 0xFF);
        assert_eq!(h.len(), 64);
    }

    #[test]
    fn ico_lists_every_image() {
        let ps: Vec<Pixmap> = [16, 24, 32, 48, 64, 128, 256]
            .iter()
            .map(|&s| render(RED, s).unwrap())
            .collect();
        let b = ico(&ps).unwrap();
        assert_eq!(&b[..4], &[0, 0, 1, 0]);
        assert_eq!(u16::from_le_bytes([b[4], b[5]]), 7);
    }

    #[test]
    fn opaque_png_has_no_alpha() {
        let b = png_opaque(&render(RED, 4).unwrap()).unwrap();
        assert_eq!(&b[1..4], b"PNG");
        assert_eq!(b[25], 2);
        assert_eq!(png(&render(RED, 4).unwrap()).unwrap()[25], 6);
    }

    #[test]
    fn external_image_files_are_not_read() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("secret.png");
        fs::write(&file, png(&render(RED, 4).unwrap()).unwrap()).unwrap();
        let svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 4 4" width="4" height="4"><image href="{}" width="4" height="4"/></svg>"#,
            file.display()
        );
        let p = render(&svg, 4).unwrap();
        assert_eq!(p.pixel(1, 1).unwrap().alpha(), 0, "the file was drawn");
    }

    #[test]
    fn opaque_png_refuses_transparent_pixels() {
        let e = png_opaque(&render(HALF, 4).unwrap()).unwrap_err();
        assert_eq!(e, "png_opaque: mark has transparent pixels");
    }
}
