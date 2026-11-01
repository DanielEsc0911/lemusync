//! The output table: every icon and pixmap file, its mark, size and format.

/// How a rendered mark is encoded.
pub enum Format {
    Png,
    PngOpaque,
    /// One ICO holding every listed size.
    Ico(&'static [u32]),
    Argb32,
}

pub struct Target {
    /// Relative to the repository root.
    pub path: String,
    pub mark: &'static str,
    pub px: u32,
    pub format: Format,
}

/// Composed marks written as SVG text: (path, mark).
pub const SVGS: [(&str, &str); 2] = [
    ("brand/out/app.svg", "app"),
    ("brand/out/glyph.svg", "glyph"),
];

pub const TRAY_STATES: [&str; 6] = [
    "synced",
    "syncing",
    "paused",
    "offline",
    "attention",
    "error",
];
pub const TRAY_SIZES: [u32; 3] = [24, 32, 48];
pub const ICO_SIZES: [u32; 7] = [16, 24, 32, 48, 64, 128, 256];

pub const IOS_DIR: &str = "app/ios/Runner/Assets.xcassets/AppIcon.appiconset";
pub const MACOS_DIR: &str = "app/macos/Runner/Assets.xcassets/AppIcon.appiconset";

const IOS: [(&str, u32); 15] = [
    ("Icon-App-20x20@1x.png", 20),
    ("Icon-App-20x20@2x.png", 40),
    ("Icon-App-20x20@3x.png", 60),
    ("Icon-App-29x29@1x.png", 29),
    ("Icon-App-29x29@2x.png", 58),
    ("Icon-App-29x29@3x.png", 87),
    ("Icon-App-40x40@1x.png", 40),
    ("Icon-App-40x40@2x.png", 80),
    ("Icon-App-40x40@3x.png", 120),
    ("Icon-App-60x60@2x.png", 120),
    ("Icon-App-60x60@3x.png", 180),
    ("Icon-App-76x76@1x.png", 76),
    ("Icon-App-76x76@2x.png", 152),
    ("Icon-App-83.5x83.5@2x.png", 167),
    ("Icon-App-1024x1024@1x.png", 1024),
];

const MACOS_SIZES: [u32; 7] = [16, 32, 64, 128, 256, 512, 1024];
const ANDROID: [(&str, u32); 5] = [
    ("mdpi", 48),
    ("hdpi", 72),
    ("xhdpi", 96),
    ("xxhdpi", 144),
    ("xxxhdpi", 192),
];
const IN_APP: [(&str, u32); 3] = [
    ("app/assets/brand/mark.png", 64),
    ("app/assets/brand/2.0x/mark.png", 128),
    ("app/assets/brand/3.0x/mark.png", 192),
];

fn target(path: String, mark: &'static str, px: u32, format: Format) -> Target {
    Target {
        path,
        mark,
        px,
        format,
    }
}

/// Every raster output.
pub fn targets() -> Vec<Target> {
    let mut t = vec![target(
        "packaging/icons/lemusync.png".into(),
        "app",
        256,
        Format::Png,
    )];
    for (density, px) in ANDROID {
        t.push(target(
            format!("app/android/app/src/main/res/mipmap-{density}/ic_launcher.png"),
            "app",
            px,
            Format::Png,
        ));
    }
    for (file, px) in IOS {
        t.push(target(
            format!("{IOS_DIR}/{file}"),
            "appSquare",
            px,
            Format::PngOpaque,
        ));
    }
    for px in MACOS_SIZES {
        t.push(target(
            format!("{MACOS_DIR}/app_icon_{px}.png"),
            "app",
            px,
            Format::Png,
        ));
    }
    t.push(target(
        "app/windows/runner/resources/app_icon.ico".into(),
        "app",
        256,
        Format::Ico(&ICO_SIZES),
    ));
    for (path, px) in IN_APP {
        t.push(target(path.into(), "glyph", px, Format::Png));
    }
    for state in TRAY_STATES {
        for px in TRAY_SIZES {
            let mark = match state {
                "synced" => "tray-synced",
                "syncing" => "tray-syncing",
                "paused" => "tray-paused",
                "offline" => "tray-offline",
                "attention" => "tray-attention",
                _ => "tray-error",
            };
            t.push(target(
                format!("crates/daemon/assets/tray/{state}-{px}.argb"),
                mark,
                px,
                Format::Argb32,
            ));
        }
    }
    t
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::path::Path;

    use super::*;
    use crate::brand::mark::{load_marks, safe_name};

    fn root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
    }

    #[test]
    fn paths_are_unique_and_safe() {
        let mut seen = BTreeSet::new();
        let svgs = SVGS.iter().map(|(p, _)| (*p).to_string());
        for p in targets().into_iter().map(|t| t.path).chain(svgs) {
            assert!(seen.insert(p.clone()), "duplicate {p}");
            assert!(!p.starts_with('/'));
            assert!(p.split('/').all(safe_name), "unsafe {p}");
        }
    }

    /// `filename` → pixel size, from a Contents.json (`size` may be `83.5x83.5`).
    fn from_contents(dir: &str) -> BTreeMap<String, u32> {
        let text = fs::read_to_string(root().join(dir).join("Contents.json")).unwrap();
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        let mut out = BTreeMap::new();
        for img in json["images"].as_array().unwrap() {
            let Some(file) = img["filename"].as_str() else {
                continue;
            };
            let size: f64 = img["size"]
                .as_str()
                .unwrap()
                .split('x')
                .next()
                .unwrap()
                .parse()
                .unwrap();
            let scale: f64 = img["scale"]
                .as_str()
                .unwrap()
                .trim_end_matches('x')
                .parse()
                .unwrap();
            let px = (size * scale).round();
            assert!((0.0..=4096.0).contains(&px));
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            // range asserted above
            let px = px as u32;
            if let Some(old) = out.insert(file.to_string(), px) {
                assert_eq!(old, px, "{file} disagrees with itself");
            }
        }
        out
    }

    fn table(dir: &str) -> BTreeMap<String, u32> {
        let prefix = format!("{dir}/");
        targets()
            .into_iter()
            .filter_map(|t| t.path.strip_prefix(&prefix).map(|f| (f.to_string(), t.px)))
            .collect()
    }

    #[test]
    fn ios_and_macos_match_their_contents_json() {
        assert_eq!(table(IOS_DIR), from_contents(IOS_DIR));
        assert_eq!(table(MACOS_DIR), from_contents(MACOS_DIR));
    }

    #[test]
    fn every_mark_used_exists() {
        let marks = load_marks(&root().join("brand/mark/marks.toml")).unwrap();
        let names = marks.names();
        for t in targets() {
            assert!(
                names.contains(&t.mark),
                "{} uses unknown mark {}",
                t.path,
                t.mark
            );
        }
        for (_, m) in SVGS {
            assert!(names.contains(&m));
        }
    }

    #[test]
    fn tray_files_cover_six_states_and_three_sizes() {
        let n = targets()
            .iter()
            .filter(|t| matches!(t.format, Format::Argb32))
            .count();
        assert_eq!(n, 18);
    }
}
