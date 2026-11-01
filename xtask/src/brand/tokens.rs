//! Parse and validate `brand/tokens.toml` into [`Tokens`].

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use super::contrast::contrast;

/// An sRGB color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

/// The semantic color roles every theme defines, in output order.
pub const ROLES: [&str; 21] = [
    "surface",
    "surfaceSunken",
    "surfaceRaised",
    "onSurface",
    "onSurfaceMuted",
    "outline",
    "bevelLight",
    "bevelDark",
    "accent",
    "onAccent",
    "focus",
    "synced",
    "onSynced",
    "syncing",
    "onSyncing",
    "attention",
    "onAttention",
    "danger",
    "onDanger",
    "offline",
    "onOffline",
];

/// The themes, in output order.
pub const THEMES: [&str; 4] = ["light", "dark", "lightHighContrast", "darkHighContrast"];

/// Type-scale entries, in output order.
pub const TYPE_STEPS: [&str; 7] = [
    "display", "headline", "title", "body", "label", "caption", "mono",
];
const FAMILIES: [&str; 3] = ["display", "body", "mono"];
pub const SPACE_KEYS: [&str; 7] = ["xs", "sm", "md", "lg", "xl", "xxl", "xxxl"];
pub const SIZE_KEYS: [&str; 8] = [
    "target",
    "iconSm",
    "iconMd",
    "iconLg",
    "bevel",
    "focusRing",
    "ringStroke",
    "mark",
];
pub const SHAPE_KEYS: [&str; 4] = ["radiusSm", "radiusMd", "radiusLg", "radiusPill"];
pub const OPACITY_KEYS: [&str; 3] = ["pressed", "hover", "disabled"];

/// One text style of the scale.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeStyle {
    pub family: String,
    pub size: f64,
    pub weight: u16,
    pub height: f64,
    pub tracking: f64,
}

/// A validated token set.
#[derive(Clone, Debug)]
pub struct Tokens {
    #[allow(dead_code)] // read by the mark generator (Task 4)
    pub palette: BTreeMap<String, Rgb>,
    /// theme → role → color
    pub themes: BTreeMap<String, BTreeMap<String, Rgb>>,
    /// `display`, `body`, `mono` → font family name
    pub families: BTreeMap<String, String>,
    pub scale: BTreeMap<String, TypeStyle>,
    pub space: BTreeMap<String, f64>,
    pub size: BTreeMap<String, f64>,
    pub shape: BTreeMap<String, f64>,
    pub opacity: BTreeMap<String, f64>,
    pub durations: BTreeMap<String, u32>,
    pub curves: BTreeMap<String, [f64; 4]>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

impl OneOrMany {
    fn items(&self) -> Vec<&str> {
        match self {
            Self::One(s) => vec![s.as_str()],
            Self::Many(v) => v.iter().map(String::as_str).collect(),
        }
    }
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum Kind {
    Text,
    Ui,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPair {
    fg: OneOrMany,
    bg: OneOrMany,
    kind: Kind,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawType {
    families: BTreeMap<String, String>,
    scale: BTreeMap<String, TypeStyle>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMotion {
    fast: u32,
    medium: u32,
    slow: u32,
    standard: [f64; 4],
    emphasized: [f64; 4],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    schema: u32,
    palette: BTreeMap<String, String>,
    themes: BTreeMap<String, BTreeMap<String, String>>,
    contrast: Vec<RawPair>,
    #[serde(rename = "type")]
    ty: RawType,
    space: BTreeMap<String, f64>,
    size: BTreeMap<String, f64>,
    shape: BTreeMap<String, f64>,
    opacity: BTreeMap<String, f64>,
    motion: RawMotion,
}

fn parse_hex(s: &str) -> Option<Rgb> {
    let h = s.strip_prefix('#')?;
    if h.len() != 6 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let b = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).ok();
    Some(Rgb(b(0)?, b(2)?, b(4)?))
}

fn check_keys<V>(
    section: &str,
    map: &BTreeMap<String, V>,
    keys: &[&str],
    errors: &mut Vec<String>,
) {
    for k in keys {
        if !map.contains_key(*k) {
            errors.push(format!("{section}: missing `{k}`"));
        }
    }
    for k in map.keys() {
        if !keys.contains(&k.as_str()) {
            errors.push(format!("{section}: unknown key `{k}`"));
        }
    }
}

impl Tokens {
    /// Reads and validates the token file.
    pub fn load(path: &Path) -> Result<Self, Vec<String>> {
        let src = fs::read_to_string(path).map_err(|e| vec![format!("{}: {e}", path.display())])?;
        Self::load_str(&src).map_err(|errs| {
            errs.into_iter()
                .map(|e| format!("{}: {e}", path.display()))
                .collect()
        })
    }

    /// Parses and validates token text.
    pub fn load_str(src: &str) -> Result<Self, Vec<String>> {
        let raw: Raw = toml::from_str(src).map_err(|e| vec![e.to_string()])?;
        let mut errors = Vec::new();
        if raw.schema != 1 {
            errors.push(format!("schema: expected 1, found {}", raw.schema));
        }
        let mut palette = BTreeMap::new();
        for (name, hex) in &raw.palette {
            match parse_hex(hex) {
                Some(c) => {
                    palette.insert(name.clone(), c);
                }
                None => errors.push(format!("palette.{name}: bad hex `{hex}` (want #RRGGBB)")),
            }
        }
        check_keys("themes", &raw.themes, &THEMES, &mut errors);
        let mut themes = BTreeMap::new();
        for (theme, roles) in &raw.themes {
            check_keys(&format!("themes.{theme}"), roles, &ROLES, &mut errors);
            let mut resolved = BTreeMap::new();
            for (role, prim) in roles {
                match palette.get(prim) {
                    Some(c) => {
                        resolved.insert(role.clone(), *c);
                    }
                    None => {
                        errors.push(format!("themes.{theme}.{role}: unknown primitive `{prim}`"));
                    }
                }
            }
            themes.insert(theme.clone(), resolved);
        }
        check_keys("type.families", &raw.ty.families, &FAMILIES, &mut errors);
        check_keys("type.scale", &raw.ty.scale, &TYPE_STEPS, &mut errors);
        for (name, style) in &raw.ty.scale {
            if !FAMILIES.contains(&style.family.as_str()) {
                errors.push(format!(
                    "type.scale.{name}: unknown family `{}`",
                    style.family
                ));
            }
        }
        check_keys("space", &raw.space, &SPACE_KEYS, &mut errors);
        check_keys("size", &raw.size, &SIZE_KEYS, &mut errors);
        check_keys("shape", &raw.shape, &SHAPE_KEYS, &mut errors);
        check_keys("opacity", &raw.opacity, &OPACITY_KEYS, &mut errors);
        if errors.is_empty() {
            check_contrast(&raw.contrast, &themes, &mut errors);
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        let m = raw.motion;
        Ok(Self {
            palette,
            themes,
            families: raw.ty.families,
            scale: raw.ty.scale,
            space: raw.space,
            size: raw.size,
            shape: raw.shape,
            opacity: raw.opacity,
            durations: BTreeMap::from([
                ("fast".to_owned(), m.fast),
                ("medium".to_owned(), m.medium),
                ("slow".to_owned(), m.slow),
            ]),
            curves: BTreeMap::from([
                ("standard".to_owned(), m.standard),
                ("emphasized".to_owned(), m.emphasized),
            ]),
        })
    }

    #[allow(dead_code)] // used by the mark generator (Task 4)
    pub fn primitive(&self, name: &str) -> Option<Rgb> {
        self.palette.get(name).copied()
    }

    /// The color of `role` in `theme`; both exist in a validated set.
    pub fn color(&self, theme: &str, role: &str) -> Rgb {
        self.themes[theme][role]
    }
}

fn check_contrast(
    pairs: &[RawPair],
    themes: &BTreeMap<String, BTreeMap<String, Rgb>>,
    errors: &mut Vec<String>,
) {
    for pair in pairs {
        for r in pair.fg.items().into_iter().chain(pair.bg.items()) {
            if !ROLES.contains(&r) {
                errors.push(format!("contrast: unknown role `{r}`"));
            }
        }
    }
    if !errors.is_empty() {
        return;
    }
    for (theme, roles) in themes {
        let high = theme.ends_with("HighContrast");
        for pair in pairs {
            let min = match (pair.kind, high) {
                (Kind::Text, false) | (Kind::Ui, true) => 4.5,
                (Kind::Text, true) => 7.0,
                (Kind::Ui, false) => 3.0,
            };
            for fg in pair.fg.items() {
                for bg in pair.bg.items() {
                    let ratio = contrast(roles[fg], roles[bg]);
                    if ratio < min {
                        errors.push(format!(
                            "contrast: {theme}: {fg} on {bg} is {ratio:.2}:1, minimum {min:.2}"
                        ));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REAL: &str = include_str!("../../../brand/tokens.toml");

    fn err_of(src: &str) -> String {
        Tokens::load_str(src).expect_err("should fail").join("\n")
    }

    #[test]
    fn real_file_loads() {
        let t = Tokens::load_str(REAL).unwrap();
        assert_eq!(t.themes.len(), 4);
        assert_eq!(t.color("light", "surface"), Rgb(0xEC, 0xE9, 0xE3));
        assert_eq!(t.primitive("bone"), Some(Rgb(0xFB, 0xFA, 0xF7)));
    }

    #[test]
    fn missing_role_names_theme_and_role() {
        let e = err_of(&REAL.replacen("focus = \"lavender-700\"\n", "", 1));
        assert!(e.contains("themes.light") && e.contains("focus"), "{e}");
    }

    #[test]
    fn extra_role_fails() {
        let e = err_of(&REAL.replacen("[themes.light]\n", "[themes.light]\nbogus = \"bone\"\n", 1));
        assert!(e.contains("themes.light") && e.contains("bogus"), "{e}");
    }

    #[test]
    fn unknown_primitive_fails() {
        let e = err_of(&REAL.replacen("surface = \"stone-100\"", "surface = \"nope\"", 1));
        assert!(
            e.contains("themes.light.surface") && e.contains("nope"),
            "{e}"
        );
    }

    #[test]
    fn bad_hex_fails() {
        let e = err_of(&REAL.replacen("#141211", "#12345", 1));
        assert!(e.contains("charcoal-950") && e.contains("#12345"), "{e}");
    }

    #[test]
    fn unknown_family_fails() {
        let e = err_of(&REAL.replacen(
            "family = \"display\", size = 40",
            "family = \"serif\", size = 40",
            1,
        ));
        assert!(
            e.contains("type.scale.display") && e.contains("serif"),
            "{e}"
        );
    }

    #[test]
    fn unknown_top_level_key_fails() {
        let e = err_of(&format!("extra = 1\n{REAL}"));
        assert!(e.contains("extra"), "{e}");
    }

    #[test]
    fn low_contrast_names_everything() {
        let e = err_of(&REAL.replacen(
            "onSurface = \"charcoal-900\"",
            "onSurface = \"stone-200\"",
            1,
        ));
        assert!(
            e.contains("light")
                && e.contains("onSurface")
                && e.contains("surface")
                && e.contains("minimum 4.50"),
            "{e}"
        );
        assert!(e.contains(':') && e.contains('.'), "{e}");
    }

    #[test]
    fn high_contrast_needs_seven() {
        let e = err_of(&REAL.replacen(
            "onSurfaceMuted = \"charcoal-700\"",
            "onSurfaceMuted = \"stone-600\"",
            1,
        ));
        assert!(
            e.contains("lightHighContrast") && e.contains("minimum 7.00"),
            "{e}"
        );
    }
}
