//! Verify the SHA-256 of every vendored file listed in `app/assets/fonts/SOURCES.md`.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};

const SOURCES: &str = "app/assets/fonts/SOURCES.md";

/// Table rows look like ``| `path` | ... | `sha256` |``; paths are relative to the SOURCES.md folder.
fn rows(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in text.lines().filter(|l| l.starts_with('|')) {
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        let (Some(first), Some(last)) = (
            cells
                .get(1)
                .and_then(|c| c.strip_prefix('`')?.strip_suffix('`')),
            cells
                .iter()
                .rev()
                .nth(1)
                .and_then(|c| c.strip_prefix('`')?.strip_suffix('`')),
        ) else {
            continue;
        };
        if last.len() == 64 && last.bytes().all(|b| b.is_ascii_hexdigit()) {
            out.push((first.to_string(), last.to_string()));
        }
    }
    out
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

pub fn check(root: &Path) -> Vec<String> {
    let file = root.join(SOURCES);
    let Ok(text) = fs::read_to_string(&file) else {
        return vec![format!("{SOURCES}: cannot read")];
    };
    let dir = file.parent().unwrap_or(root);
    let rows = rows(&text);
    if rows.is_empty() {
        return vec![format!("{SOURCES}: no hash rows found")];
    }
    let mut errors = Vec::new();
    for (path, want) in rows {
        match fs::read(dir.join(&path)) {
            Err(e) => errors.push(format!("{SOURCES}: {path}: {e}")),
            Ok(bytes) => {
                let got = hex(&Sha256::digest(&bytes));
                if got != want {
                    errors.push(format!(
                        "{SOURCES}: {path}: SHA-256 is {got}, listed {want}"
                    ));
                }
            }
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rows_and_checks_hashes() {
        let t = tempfile::tempdir().unwrap();
        let dir = t.path().join("app/assets/fonts");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("a.txt"), b"abc").unwrap();
        let good = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        let md = |h: &str| {
            format!(
                "| File | x | URL | SHA-256 |\n|---|---|---|---|\n| `a.txt` | `a` | https://x | `{h}` |\n"
            )
        };
        fs::write(dir.join("SOURCES.md"), md(good)).unwrap();
        assert_eq!(check(t.path()).len(), 0);
        fs::write(dir.join("SOURCES.md"), md(&"0".repeat(64))).unwrap();
        assert_eq!(check(t.path()).len(), 1);
    }
}
