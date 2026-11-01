//! Scan `app/lib` for hard-coded design literals.
//!
//! The scan works on whole-file text, so arguments wrapped by the formatter are still seen.

use std::fs;
use std::path::Path;

const ROOT: &str = "app/lib";
const SKIP: [&str; 3] = [
    "app/lib/design/tokens",
    "app/lib/l10n/generated",
    "app/lib/bridge",
];
const MARK: &str = "// brand-literal-ok:";

/// Constructors whose arguments are design values.
const ARG_PREFIXES: [&str; 11] = [
    "EdgeInsets.",
    "EdgeInsetsDirectional.",
    "BorderRadius.",
    "BorderRadiusDirectional.",
    "Radius.",
    "Size(",
    "Size.",
    "SizedBox(",
    "SizedBox.",
    "BoxConstraints(",
    "BorderSide(",
];

/// Named arguments that carry a design value.
const NAMED: [&str; 24] = [
    "fontSize",
    "iconSize",
    "letterSpacing",
    "wordSpacing",
    "elevation",
    "strokeWidth",
    "thickness",
    "indent",
    "endIndent",
    "opacity",
    "alpha",
    "width",
    "height",
    "dimension",
    "radius",
    "blurRadius",
    "spreadRadius",
    "minWidth",
    "maxWidth",
    "minHeight",
    "maxHeight",
    "spacing",
    "runSpacing",
    "size",
];

/// Substrings that are always a literal.
const ALWAYS: [&str; 10] = [
    "Color.fromARGB(",
    "Color.fromRGBO(",
    "Color.from(",
    "CupertinoColors.",
    ".withAlpha(",
    ".withOpacity(",
    "Curves.",
    "Cubic(",
    "Duration(",
    "fontWeight: FontWeight.w",
];

fn is_ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn prev_char(text: &str, i: usize) -> Option<char> {
    text[..i].chars().next_back()
}

fn starts_with_digit(s: &str) -> bool {
    s.trim_start()
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_digit())
}

/// The numeric literal at the start of `s`.
fn number(s: &str) -> &str {
    let end = s
        .char_indices()
        .find(|&(_, c)| !(c.is_ascii_alphanumeric() || c == '.' || c == '_'))
        .map_or(s.len(), |(i, _)| i);
    &s[..end]
}

fn is_zero(lit: &str) -> bool {
    lit == "0" || lit == "0.0"
}

/// Text inside the parentheses that open at the start of `s` (up to the matching `)`).
fn args(s: &str) -> Option<&str> {
    let mut depth = 0usize;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(&s[1..i]);
                }
            }
            _ => {}
        }
    }
    None
}

/// Whether `body` holds a numeric literal other than zero.
fn has_design_number(body: &str) -> bool {
    let mut prev: Option<char> = None;
    let mut it = body.char_indices().peekable();
    while let Some((i, c)) = it.next() {
        if c.is_ascii_digit() && !prev.is_some_and(|p| is_ident(p) || p == '.') {
            let lit = number(&body[i..]);
            if !is_zero(lit) {
                return true;
            }
            for _ in 1..lit.chars().count() {
                it.next();
            }
            prev = lit.chars().next_back();
            continue;
        }
        prev = Some(c);
    }
    false
}

/// Byte offsets where a hard-coded design literal starts.
fn matches(text: &str) -> Vec<usize> {
    let mut out = Vec::new();
    for prefix in ARG_PREFIXES {
        for (i, _) in text.match_indices(prefix) {
            if prev_char(text, i).is_some_and(is_ident) {
                continue;
            }
            let rest = if prefix.ends_with('(') {
                &text[i + prefix.len() - 1..]
            } else {
                let after = &text[i + prefix.len()..];
                &after[after.find(|c: char| !is_ident(c)).unwrap_or(after.len())..]
            };
            if rest.starts_with('(') && args(rest).is_some_and(has_design_number) {
                out.push(i);
            }
        }
    }
    for name in NAMED {
        for (i, _) in text.match_indices(name) {
            if prev_char(text, i).is_some_and(is_ident) {
                continue;
            }
            let Some(rest) = text[i + name.len()..].strip_prefix(':') else {
                continue;
            };
            let rest = rest.trim_start();
            let rest = rest.strip_prefix('-').unwrap_or(rest);
            if starts_with_digit(rest) && !is_zero(number(rest)) {
                out.push(i);
            }
        }
    }
    for (i, _) in text.match_indices("Color(") {
        if starts_with_digit(&text[i + 6..]) {
            out.push(i);
        }
    }
    for (i, _) in text.match_indices(".withValues(alpha:") {
        if starts_with_digit(&text[i + 18..]) {
            out.push(i);
        }
    }
    for (i, _) in text.match_indices("fontFamily:") {
        if text[i + 11..].trim_start().starts_with(['\'', '"']) {
            out.push(i);
        }
    }
    for needle in ALWAYS {
        out.extend(text.match_indices(needle).map(|(i, _)| i));
    }
    // `Colors.x` except `Colors.transparent`; not `LsColors.` or any other prefixed name.
    for (i, _) in text.match_indices("Colors.") {
        let word: String = text[i + 7..].chars().take_while(|c| is_ident(*c)).collect();
        if !prev_char(text, i).is_some_and(is_ident) && word != "transparent" {
            out.push(i);
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

fn escaped(line: &str) -> bool {
    line.rsplit_once(MARK)
        .is_some_and(|(_, r)| !r.trim().is_empty())
}

/// `line: text` for every unescaped literal in `text`.
fn scan_text(text: &str) -> Vec<(usize, String)> {
    let mut hits: Vec<(usize, String)> = Vec::new();
    for off in matches(text) {
        let line_no = text[..off].matches('\n').count() + 1;
        if hits.last().is_some_and(|(n, _)| *n == line_no) {
            continue;
        }
        let line = text.lines().nth(line_no - 1).unwrap_or_default();
        if !escaped(line) {
            hits.push((line_no, line.trim().to_owned()));
        }
    }
    hits
}

/// What the scan found.
#[derive(Debug, Default)]
pub struct Scan {
    /// `file:line: text` for every hard-coded design literal outside the generated tokens.
    pub hits: Vec<String>,
    /// Folders or files that could not be read.
    pub errors: Vec<String>,
}

fn walk(root: &Path, dir: &Path, scan: &mut Scan) {
    let rd = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(e) => {
            scan.errors.push(format!("{}: {e}", dir.display()));
            return;
        }
    };
    let mut entries = Vec::new();
    for entry in rd {
        match entry {
            Ok(e) => entries.push(e),
            Err(e) => scan.errors.push(format!("{}: {e}", dir.display())),
        }
    }
    entries.sort_by_key(fs::DirEntry::path);
    for entry in entries {
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if SKIP.contains(&rel.as_str()) {
            continue;
        }
        let kind = match entry.file_type() {
            Ok(k) => k,
            Err(e) => {
                scan.errors.push(format!("{rel}: {e}"));
                continue;
            }
        };
        if kind.is_dir() {
            walk(root, &path, scan);
        } else if kind.is_file() && path.extension().is_some_and(|e| e == "dart") {
            match fs::read_to_string(&path) {
                Ok(text) => {
                    for (n, line) in scan_text(&text) {
                        scan.hits.push(format!("{rel}:{n}: {line}"));
                    }
                }
                Err(e) => scan.errors.push(format!("{rel}: {e}")),
            }
        }
    }
}

/// Scans every Dart file under `app/lib`.
pub fn scan(root: &Path) -> Scan {
    let mut scan = Scan::default();
    walk(root, &root.join(ROOT), &mut scan);
    scan
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write(root: &Path, rel: &str, text: &str) {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
    }

    fn hits_of(code: &str) -> Vec<(usize, String)> {
        scan_text(code)
    }

    fn flagged(code: &str) {
        assert!(!hits_of(code).is_empty(), "should flag: {code}");
    }

    fn clean(code: &str) {
        assert!(hits_of(code).is_empty(), "should allow: {code}");
    }

    #[test]
    fn flags_each_pattern() {
        for code in [
            "final a = Color(0xFF000000);",
            "final a = Color(0XFF000000);",
            "final a = Color(4278190080);",
            "final a = Color.fromARGB(255, 0, 0, 0);",
            "final a = Color.fromRGBO(0, 0, 0, 1);",
            "final a = Color.from(alpha: 1, red: 0, green: 0, blue: 0);",
            "final b = Colors.red;",
            "final b = CupertinoColors.activeBlue;",
            "final c = EdgeInsets.all(16);",
            "final c = EdgeInsets.symmetric(horizontal: 8, vertical: LsSpace.md);",
            "final c = EdgeInsetsDirectional.only(start: 4);",
            "final c = BorderRadius.circular(8);",
            "final c = BorderRadiusDirectional.circular(8);",
            "final c = Radius.circular(4);",
            "final c = Size(48, 48);",
            "final c = Size.square(48);",
            "final c = SizedBox(key: k, width: 8);",
            "final c = SizedBox.square(dimension: 8);",
            "final c = BoxConstraints(maxWidth: 400);",
            "final c = BorderSide(width: 2);",
            "final c = TextStyle(fontSize: 14);",
            "final c = Container(width: 8, height: 8);",
            "final c = Icon(size: 24);",
            "final c = Icon(iconSize: -3);",
            "final c = x(elevation: 2, thickness: 1);",
            "final c = c.withAlpha(128);",
            "final c = c.withOpacity(0.5);",
            "final c = c.withValues(alpha: 0.5);",
            "final c = Curves.easeIn;",
            "final c = Cubic(0.1, 0.2, 0.3, 0.4);",
            "final c = Duration(milliseconds: 5);",
            "final c = TextStyle(fontFamily: 'Inter');",
            "final c = TextStyle(fontWeight: FontWeight.w700);",
        ] {
            flagged(code);
        }
    }

    #[test]
    fn flags_wrapped_arguments_on_the_start_line() {
        let hits = hits_of(
            "a;\nfinal c = EdgeInsets.all(\n  16,\n);\nfinal d = SizedBox(\n  width:\n    8);\n",
        );
        let lines: Vec<usize> = hits.iter().map(|h| h.0).collect();
        assert_eq!(lines, vec![2, 5, 6], "{hits:?}");
    }

    #[test]
    fn allows_tokens_and_zero() {
        for code in [
            "final a = LsSpace.md;",
            "final a = EdgeInsets.all(LsSpace.md);",
            "final a = EdgeInsets.zero;",
            "final a = EdgeInsets.all(0);",
            "final a = SizedBox(width: 0.0);",
            "final a = Size.square(LsSize.target);",
            "final a = LsColors.light;",
            "final a = Colors.transparent;",
            "final a = Card(elevation: 0);",
            "final a = Color(value);",
            "final a = x(width: LsSize.mark, height: w2);",
            "final a = Radius.circular(LsShape.radiusMd);",
            "final a = SizedBox(width: LsSpace.x2);",
            "final a = Foo(opacity: o);",
        ] {
            clean(code);
        }
    }

    #[test]
    fn marker_needs_a_reason_on_the_start_line() {
        clean("final c = EdgeInsets.all( // brand-literal-ok: wrapped\n  16,\n);\n");
        flagged("final c = EdgeInsets.all(\n  16, // brand-literal-ok: wrong line\n);\n");
        flagged("final c = Duration(seconds: 1); // brand-literal-ok:   \n");
        clean("final c = Duration(seconds: 1); // brand-literal-ok: test clock\n");
    }

    #[test]
    fn skips_generated_folders_and_flags_files() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "app/lib/x.dart", "final a = Colors.red;\n");
        write(
            dir.path(),
            "app/lib/design/tokens/c.g.dart",
            "Color(0xFF000000)\n",
        );
        write(dir.path(), "app/lib/l10n/generated/l.dart", "Colors.red\n");
        write(dir.path(), "app/lib/bridge/b.dart", "Colors.red\n");
        let s = scan(dir.path());
        assert_eq!(s.hits, vec!["app/lib/x.dart:1: final a = Colors.red;"]);
        assert_eq!(s.errors.len(), 0);
    }

    #[test]
    fn missing_app_lib_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(scan(dir.path()).errors.len(), 1);
    }

    #[test]
    fn non_utf8_dart_file_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("app/lib/bad.dart");
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, [0xff, 0xfe, 0x00]).unwrap();
        let s = scan(dir.path());
        assert_eq!(s.errors.len(), 1, "{s:?}");
        assert!(s.errors[0].contains("bad.dart"));
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_are_not_followed() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "outside/o.dart", "final a = Colors.red;\n");
        fs::create_dir_all(dir.path().join("app/lib")).unwrap();
        std::os::unix::fs::symlink(dir.path().join("outside"), dir.path().join("app/lib/link"))
            .unwrap();
        std::os::unix::fs::symlink(
            dir.path().join("outside/o.dart"),
            dir.path().join("app/lib/l.dart"),
        )
        .unwrap();
        let s = scan(dir.path());
        assert_eq!(s.hits.len() + s.errors.len(), 0, "{s:?}");
    }
}
