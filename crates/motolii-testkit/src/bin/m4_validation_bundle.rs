use std::path::PathBuf;
use std::process::Command;

use motolii_testkit::perf::{
    write_m4_validation_bundle, M4ValidationContext, M4_VALIDATION_CONTEXT_SCHEMA_VERSION,
};

#[derive(Debug, thiserror::Error)]
enum BundleError {
    #[error(
        "usage: m4_validation_bundle <output-directory> <machine-label> <intended-persona> <ac|battery> <power-mode> <display-width-px> <display-height-px>"
    )]
    Usage,
    #[error("machine label, intended persona, and power mode must be non-empty")]
    EmptyContext,
    #[error("power source must be ac or battery")]
    PowerSource,
    #[error("display dimensions must be positive integers")]
    DisplayDimensions,
    #[error("context labels and numeric arguments must be valid UTF-8")]
    NonUtf8Context,
}

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
    let mut args = std::env::args_os().skip(1);
    let output_dir = PathBuf::from(args.next().ok_or(BundleError::Usage)?);
    let mut next_string = || {
        args.next()
            .ok_or(BundleError::Usage)?
            .into_string()
            .map_err(|_| BundleError::NonUtf8Context)
    };
    let machine_label = next_string()?;
    let intended_persona = next_string()?;
    let power_source = next_string()?;
    let power_mode = next_string()?;
    let display_width_px = args
        .next()
        .ok_or(BundleError::Usage)?
        .into_string()
        .map_err(|_| BundleError::NonUtf8Context)?
        .parse()
        .map_err(|_| BundleError::DisplayDimensions)?;
    let display_height_px = args
        .next()
        .ok_or(BundleError::Usage)?
        .into_string()
        .map_err(|_| BundleError::NonUtf8Context)?
        .parse()
        .map_err(|_| BundleError::DisplayDimensions)?;
    if args.next().is_some() {
        return Err(BundleError::Usage.into());
    }
    if machine_label.is_empty() || intended_persona.is_empty() || power_mode.is_empty() {
        return Err(BundleError::EmptyContext.into());
    }
    if !matches!(power_source.as_str(), "ac" | "battery") {
        return Err(BundleError::PowerSource.into());
    }
    if display_width_px == 0 || display_height_px == 0 {
        return Err(BundleError::DisplayDimensions.into());
    }
    let context = M4ValidationContext {
        schema_version: M4_VALIDATION_CONTEXT_SCHEMA_VERSION,
        machine_label,
        intended_persona,
        power_source,
        power_mode,
        display_width_px,
        display_height_px,
    };
    let manifest = write_m4_validation_bundle(&output_dir, repository_revision(), &context)?;
    println!("{}", output_dir.join("manifest.json").display());
    println!("commands={}", manifest.commands.len());
    println!("external_gates={}", manifest.external_gates.len());
    Ok(())
}
