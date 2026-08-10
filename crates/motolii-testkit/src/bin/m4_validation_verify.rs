use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use motolii_testkit::m4_validation::verify_m4_validation_bundle;

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
    let Some(output_arg) = std::env::args_os().nth(1) else {
        eprintln!("usage: m4_validation_verify <bundle-directory>");
        return ExitCode::FAILURE;
    };
    if std::env::args_os().nth(2).is_some() {
        eprintln!("usage: m4_validation_verify <bundle-directory>");
        return ExitCode::FAILURE;
    }
    let output_input = PathBuf::from(output_arg);
    let output_dir = match std::fs::canonicalize(&output_input) {
        Ok(path) => path,
        Err(error) => {
            eprintln!(
                "bundle directory does not exist or cannot be resolved: {}: {error}",
                output_input.display()
            );
            return ExitCode::FAILURE;
        }
    };
    let root = repository_root();
    if !repository_is_clean(&root) {
        eprintln!(
            "repository has tracked, staged, or untracked changes; verification requires a clean commit"
        );
        return ExitCode::FAILURE;
    }
    let Some(revision) = repository_revision(&root) else {
        eprintln!("repository HEAD is unavailable");
        return ExitCode::FAILURE;
    };
    match verify_m4_validation_bundle(&output_dir, &revision) {
        Ok(report) => {
            let json = serde_json::to_string_pretty(&report).expect("serialize verifier report");
            println!("{json}");
            if report.local_evidence_valid {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
