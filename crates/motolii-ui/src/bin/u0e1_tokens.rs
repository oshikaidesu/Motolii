use std::process;

#[path = "../tokens/mod.rs"]
mod tokens;

fn main() {
    let manifest_dir = tokens::manifest_dir();
    let command = std::env::args().nth(1);
    let result = match command.as_deref() {
        Some("check") => tokens::check(&manifest_dir),
        Some("write") => tokens::write_checked_in(&manifest_dir),
        _ => {
            eprintln!("usage: u0e1-tokens check|write");
            process::exit(2);
        }
    };

    if let Err(err) = result {
        eprintln!("{err}");
        process::exit(1);
    }
}
