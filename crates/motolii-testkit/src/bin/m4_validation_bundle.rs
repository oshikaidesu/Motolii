use std::path::PathBuf;
use std::process::Command;

use motolii_testkit::perf::write_m4_validation_bundle;

fn repository_revision() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_owned())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/m4-validation"));
    let manifest = write_m4_validation_bundle(&output_dir, repository_revision())?;
    println!("{}", output_dir.join("manifest.json").display());
    println!("commands={}", manifest.commands.len());
    println!("external_gates={}", manifest.external_gates.len());
    Ok(())
}
