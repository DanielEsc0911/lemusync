//! `brand [--check]`: generate every brand output from `brand/`, or verify they are current.

mod contrast;
mod dart;
mod literals;
mod output;
mod tokens;

use std::path::Path;

use tokens::Tokens;

const TOKENS: &str = "brand/tokens.toml";

pub fn run(root: &Path, check: bool) -> Result<(), Vec<String>> {
    let tokens = Tokens::load(&root.join(TOKENS))?;
    let outs = dart::emit(&tokens);
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
    errors.extend(literals::scan(root).into_iter().map(|hit| {
        format!("{hit}: hard-coded design literal (use a token, or end the line with `// brand-literal-ok: <reason>`)")
    }));
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
