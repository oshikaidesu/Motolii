use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: khronos_corpus <GeneratedAssets root>")?;
    let positive = inspect(&root.join("Positive"), true)?;
    let negative = inspect(&root.join("Negative"), false)?;
    println!(
        "positive: total={} core_accepted={} core_rejected={} required_extension_cases={}",
        positive.total, positive.accepted, positive.rejected, positive.required_extension_cases
    );
    println!(
        "negative: total={} accepted={} rejected={}",
        negative.total, negative.accepted, negative.rejected
    );
    for failure in positive.failures.iter().take(20) {
        eprintln!("unexpected positive rejection: {failure}");
    }
    if positive.rejected != 0 {
        return Err("gltf rejected Khronos core positive fixtures".into());
    }
    Ok(())
}

#[derive(Default)]
struct CorpusResult {
    total: usize,
    accepted: usize,
    rejected: usize,
    required_extension_cases: usize,
    failures: Vec<String>,
}

fn inspect(root: &Path, record_failures: bool) -> Result<CorpusResult, std::io::Error> {
    let mut paths = Vec::new();
    collect_gltf(root, &mut paths)?;
    paths.sort();
    let mut result = CorpusResult::default();
    for path in paths {
        result.total += 1;
        let bytes = fs::read(&path)?;
        if gltf::Gltf::from_slice_without_validation(&bytes)
            .is_ok_and(|asset| asset.extensions_required().next().is_some())
        {
            result.required_extension_cases += 1;
            continue;
        }
        match gltf::Gltf::open(&path) {
            Ok(_) => result.accepted += 1,
            Err(error) => {
                result.rejected += 1;
                if record_failures {
                    result.failures.push(format!("{}: {error}", path.display()));
                }
            }
        }
    }
    Ok(result)
}

fn collect_gltf(root: &Path, output: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_gltf(&path, output)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension == "gltf")
        {
            output.push(path);
        }
    }
    Ok(())
}
