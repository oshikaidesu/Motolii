use std::{env, process::ExitCode};

use g0_9_windowed_timeline::{FaceDescriptor, FixtureFont};
use serde::Serialize;

#[derive(Serialize)]
struct PreflightOutput {
    family: String,
    width: String,
    style: String,
    weight: f32,
    face_index: u32,
    coverage_codepoint_count: usize,
    font_sha256: String,
    glyph_digest: String,
    run_counts: Vec<RunCount>,
}

#[derive(Serialize)]
struct RunCount {
    label: String,
    glyph_count: usize,
}

#[derive(Debug, thiserror::Error, PartialEq)]
enum PreflightInputError {
    #[error("provide exactly one font descriptor: a single argument or G0_9_CJK_FACE")]
    MissingOrAmbiguous,
    #[error("only one positional font descriptor is accepted")]
    TooManyArguments,
    #[error(transparent)]
    Descriptor(#[from] g0_9_windowed_timeline::DescriptorError),
}

fn descriptor_input(
    args: &[String],
    environment: Option<&str>,
) -> Result<String, PreflightInputError> {
    match (args, environment) {
        ([], None) | ([_], Some(_)) => Err(PreflightInputError::MissingOrAmbiguous),
        ([value], None) => Ok(value.clone()),
        ([], Some(value)) => Ok(value.to_owned()),
        _ => Err(PreflightInputError::TooManyArguments),
    }
}

fn main() -> ExitCode {
    let args: Vec<_> = env::args().skip(1).collect();
    let environment = env::var("G0_9_CJK_FACE").ok();
    let descriptor = match descriptor_input(&args, environment.as_deref())
        .and_then(|input| FaceDescriptor::parse(&input).map_err(PreflightInputError::from))
    {
        Ok(descriptor) => descriptor,
        Err(error) => {
            eprintln!("g0_9_font_preflight: {error}");
            return ExitCode::FAILURE;
        }
    };
    let fixture = match FixtureFont::build(descriptor) {
        Ok(fixture) => fixture,
        Err(error) => {
            eprintln!("g0_9_font_preflight: {error}");
            return ExitCode::FAILURE;
        }
    };
    let output = PreflightOutput {
        family: fixture.descriptor.family.clone(),
        width: fixture.descriptor.width.clone(),
        style: fixture.descriptor.style.clone(),
        weight: fixture.descriptor.weight,
        face_index: fixture.face_index,
        coverage_codepoint_count: fixture.coverage_codepoint_count,
        font_sha256: fixture.font_sha256.clone(),
        glyph_digest: fixture.glyph_digest.clone(),
        run_counts: fixture
            .run_counts()
            .into_iter()
            .map(|(label, glyph_count)| RunCount {
                label: label.to_owned(),
                glyph_count,
            })
            .collect(),
    };
    match serde_json::to_string(&output) {
        Ok(json) => {
            println!("{json}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("g0_9_font_preflight: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_missing_and_ambiguous_descriptor_sources() {
        assert_eq!(
            descriptor_input(&[], None),
            Err(PreflightInputError::MissingOrAmbiguous)
        );
        assert_eq!(
            descriptor_input(&["face".to_owned()], Some("face")),
            Err(PreflightInputError::MissingOrAmbiguous)
        );
    }

    #[test]
    fn accepts_exactly_one_source() {
        assert_eq!(
            descriptor_input(&["face".to_owned()], None),
            Ok("face".to_owned())
        );
        assert_eq!(descriptor_input(&[], Some("face")), Ok("face".to_owned()));
    }
}
