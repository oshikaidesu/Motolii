use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use motolii_testkit::m4_validation::compare_m4_validation_bundles;

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn repository_revision(root: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", "HEAD"])
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_owned())
}

fn repository_is_clean(root: &Path) -> bool {
    Command::new("git")
        .args(["status", "--porcelain=v1", "--untracked-files=normal"])
        .current_dir(root)
        .output()
        .is_ok_and(|output| output.status.success() && output.stdout.is_empty())
}

fn main() -> ExitCode {
    let output_inputs: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if output_inputs.len() < 2 {
        eprintln!("usage: m4_validation_compare <bundle-directory> <bundle-directory> [...]");
        return ExitCode::FAILURE;
    }
    let mut output_dirs = Vec::with_capacity(output_inputs.len());
    for input in output_inputs {
        match std::fs::canonicalize(&input) {
            Ok(path) => output_dirs.push(path),
            Err(error) => {
                eprintln!(
                    "bundle directory does not exist or cannot be resolved: {}: {error}",
                    input.display()
                );
                return ExitCode::FAILURE;
            }
        }
    }
    let root = repository_root();
    if !repository_is_clean(&root) {
        eprintln!("repository has changes; comparison requires a clean commit");
        return ExitCode::FAILURE;
    }
    let Some(revision) = repository_revision(&root) else {
        eprintln!("repository HEAD is unavailable");
        return ExitCode::FAILURE;
    };
    match compare_m4_validation_bundles(&output_dirs, &revision) {
        Ok(matrix) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&matrix).expect("serialize comparison matrix")
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
