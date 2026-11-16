//! Repository checks, run as `cargo xtask <task>`.

mod brand;
mod budgets;
mod docs;
mod i18n;
mod layers;
mod metadata;
mod version;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let root = repo_root();
    let task = std::env::args().nth(1).unwrap_or_default();
    let result = match task.as_str() {
        "docs-check" => docs::check(&root),
        "brand" => {
            brand_check_flag(std::env::args().skip(2)).and_then(|check| brand::run(&root, check))
        }
        "layers" => layers::check(&root),
        "i18n-check" => i18n::check(&root),
        "budgets" => budgets::measure(&root, &std::env::args().nth(2).unwrap_or_default()),
        "budgets-check" => budgets::check(&root),
        "version-check" => version::check(&root, &std::env::args().nth(2).unwrap_or_default()),
        _ => Err(vec![format!(
            "unknown task `{task}`; available: brand [--check], docs-check, layers, i18n-check, version-check <tag>, budgets [idle-secs], budgets-check"
        )]),
    };
    match result {
        Ok(()) => {
            println!("xtask {task}: ok");
            ExitCode::SUCCESS
        }
        Err(errors) => {
            for error in &errors {
                eprintln!("error: {error}");
            }
            ExitCode::FAILURE
        }
    }
}

/// `brand` takes no argument or exactly `--check`.
fn brand_check_flag(mut args: impl Iterator<Item = String>) -> Result<bool, Vec<String>> {
    match (args.next().as_deref(), args.next()) {
        (None, _) => Ok(false),
        (Some("--check"), None) => Ok(true),
        _ => Err(vec!["usage: cargo xtask brand [--check]".to_owned()]),
    }
}

/// The repository root: the parent of this crate's directory.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use super::brand_check_flag;

    fn flag(args: &[&str]) -> Result<bool, Vec<String>> {
        brand_check_flag(args.iter().map(|a| (*a).to_owned()))
    }

    #[test]
    fn brand_accepts_nothing_or_check() {
        assert_eq!(flag(&[]), Ok(false));
        assert_eq!(flag(&["--check"]), Ok(true));
    }

    #[test]
    fn brand_rejects_anything_else() {
        assert!(flag(&["--chek"]).is_err());
        assert!(flag(&["--check", "x"]).is_err());
        assert!(flag(&["x"]).is_err());
    }
}
