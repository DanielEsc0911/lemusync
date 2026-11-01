//! Scan `app/lib` for hard-coded design literals.

use std::fs;
use std::path::Path;

const ROOT: &str = "app/lib";
const SKIP: [&str; 3] = [
    "app/lib/design/tokens",
    "app/lib/l10n/generated",
    "app/lib/bridge",
];
const MARK: &str = "// brand-literal-ok:";

fn starts_with_digit(s: &str) -> bool {
    s.trim_start()
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_digit())
}

fn is_ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Byte offsets of every occurrence of `needle` in `line`.
fn finds<'a>(line: &'a str, needle: &'a str) -> impl Iterator<Item = usize> + 'a {
    line.match_indices(needle).map(|(i, _)| i)
}

fn after<'a>(line: &'a str, needle: &'a str) -> impl Iterator<Item = &'a str> + 'a {
    finds(line, needle).map(move |i| &line[i + needle.len()..])
}

fn hits_line(line: &str) -> bool {
    if line
        .rsplit_once(MARK)
        .is_some_and(|(_, r)| !r.trim().is_empty())
    {
        return false;
    }
    if line.contains("Color(0x") || line.contains("Duration(") || line.contains(".withOpacity(") {
        return true;
    }
    // `Colors.x` except `Colors.transparent`; not `LsColors.` or any other prefixed name.
    for i in finds(line, "Colors.") {
        let boundary = !line[..i].chars().next_back().is_some_and(is_ident);
        let word: String = line[i + 7..].chars().take_while(|c| is_ident(*c)).collect();
        if boundary && word != "transparent" {
            return true;
        }
    }
    if after(line, "fontFamily:").any(|r| r.trim_start().starts_with(['\'', '"'])) {
        return true;
    }
    if after(line, "Radius.circular(").any(starts_with_digit) {
        return true;
    }
    for rest in after(line, "EdgeInsets.") {
        let name: String = rest.chars().take_while(char::is_ascii_alphabetic).collect();
        if !name.is_empty()
            && rest[name.len()..]
                .strip_prefix('(')
                .is_some_and(starts_with_digit)
        {
            return true;
        }
    }
    for rest in after(line, "SizedBox(") {
        for key in ["width:", "height:"] {
            if rest.strip_prefix(key).is_some_and(starts_with_digit) {
                return true;
            }
        }
    }
    after(line, "size:").any(starts_with_digit)
        || after(line, ".withValues(alpha:").any(starts_with_digit)
}

fn walk(root: &Path, dir: &Path, hits: &mut Vec<String>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = rd.flatten().map(|e| e.path()).collect();
    entries.sort();
    for path in entries {
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if SKIP.contains(&rel.as_str()) {
            continue;
        }
        if path.is_dir() {
            walk(root, &path, hits);
        } else if path.extension().is_some_and(|e| e == "dart") {
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            for (n, line) in text.lines().enumerate() {
                if hits_line(line) {
                    hits.push(format!("{rel}:{}: {}", n + 1, line.trim()));
                }
            }
        }
    }
}

/// `file:line: text` for every hard-coded design literal outside the generated tokens.
pub fn scan(root: &Path) -> Vec<String> {
    let mut hits = Vec::new();
    walk(root, &root.join(ROOT), &mut hits);
    hits
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

    #[test]
    fn flags_the_four_literals() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "app/lib/x.dart",
            "final a = Color(0xFF000000);\nfinal b = Colors.red;\nfinal c = EdgeInsets.all(16);\nfinal d = Icon(size: 24);\n",
        );
        let hits = scan(dir.path());
        assert_eq!(hits.len(), 4, "{hits:?}");
        assert!(hits[0].starts_with("app/lib/x.dart:1: "), "{hits:?}");
    }

    #[test]
    fn allows_transparent_tokens_and_marked_lines() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "app/lib/ok.dart",
            "final a = Colors.transparent;\nfinal b = LsColors.light;\nfinal c = EdgeInsets.all(0); // brand-literal-ok: zero\nfinal d = Duration(seconds: 1); // brand-literal-ok: test clock\n",
        );
        write(
            dir.path(),
            "app/lib/design/tokens/colors.g.dart",
            "Color(0xFF000000)\n",
        );
        write(dir.path(), "app/lib/l10n/generated/l.dart", "Colors.red\n");
        write(dir.path(), "app/lib/bridge/b.dart", "Colors.red\n");
        assert_eq!(scan(dir.path()), Vec::<String>::new());
    }

    #[test]
    fn flags_remaining_patterns() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "app/lib/y.dart",
            "a(fontFamily: 'Inter');\nb(Duration(milliseconds: 5));\nc(BorderRadius.circular(8));\nd(SizedBox(width: 8));\ne(c.withValues(alpha: 0.5));\nf(c.withOpacity(0.5));\ng(Radius.circular(4));\n",
        );
        assert_eq!(scan(dir.path()).len(), 7);
    }
}
