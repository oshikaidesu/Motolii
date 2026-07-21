use motolii_ui_token_gen::{check_dir, generate_to_dir, ThemeSource};
use std::path::PathBuf;

fn main() {
    if let Err(error) = run(std::env::args().skip(1).collect()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let operation = args.first().ok_or("expected `generate` or `check`")?;
    if operation != "generate" && operation != "check" {
        return Err(format!("unknown operation `{operation}`").into());
    }
    let mut themes = Vec::new();
    let mut out_dir = None;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--theme" => {
                let specification = args.get(index + 1).ok_or("missing --theme value")?;
                let (id, path) = specification
                    .split_once('=')
                    .ok_or("--theme must be <id>=<path>")?;
                themes.push(ThemeSource {
                    id: id.to_owned(),
                    bytes: std::fs::read(path)?,
                });
                index += 2;
            }
            "--out-dir" => {
                if out_dir.is_some() {
                    return Err("duplicate --out-dir".into());
                }
                out_dir = Some(PathBuf::from(
                    args.get(index + 1).ok_or("missing --out-dir value")?,
                ));
                index += 2;
            }
            argument => return Err(format!("unknown argument `{argument}`").into()),
        }
    }
    let out_dir = out_dir.ok_or("missing --out-dir")?;
    match operation.as_str() {
        "generate" => generate_to_dir(themes, &out_dir)?,
        "check" => check_dir(themes, &out_dir)?,
        _ => unreachable!(),
    }
    Ok(())
}
