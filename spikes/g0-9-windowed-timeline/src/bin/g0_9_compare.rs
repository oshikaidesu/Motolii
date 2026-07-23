use std::{env, fs, path::PathBuf, process::ExitCode};

use g0_9_windowed_timeline::{write_comparison_artifact, RawReport};

fn read_report(path: &PathBuf) -> Result<RawReport, String> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("{}: {error}", path.display()))
}

fn main() -> ExitCode {
    let arguments: Vec<_> = env::args_os().skip(1).collect();
    if arguments.len() != 3 {
        eprintln!("usage: g0_9_compare <direct-raw.json> <egui-raw.json> <comparison.json>");
        return ExitCode::FAILURE;
    }
    let direct = PathBuf::from(&arguments[0]);
    let egui = PathBuf::from(&arguments[1]);
    let output = PathBuf::from(&arguments[2]);
    let direct = match read_report(&direct) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("g0_9_compare: {error}");
            return ExitCode::FAILURE;
        }
    };
    let egui = match read_report(&egui) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("g0_9_compare: {error}");
            return ExitCode::FAILURE;
        }
    };
    match write_comparison_artifact(&output, direct, egui) {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("g0_9_compare: {error}");
            ExitCode::FAILURE
        }
    }
}
