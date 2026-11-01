//! Repository checks, run as `cargo xtask <task>`.

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
        "layers" => layers::check(&root),
        "i18n-check" => i18n::check(&root),
        "budgets" => budgets::measure(&root, &std::env::args().nth(2).unwrap_or_default()),
        "budgets-check" => budgets::check(&root),
        "version-check" => version::check(&root, &std::env::args().nth(2).unwrap_or_default()),
        _ => Err(vec![format!(
            "unknown task `{task}`; available: docs-check, layers, i18n-check, version-check <tag>, budgets [idle-secs], budgets-check"
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

/// The repository root: the parent of this crate's directory.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}
