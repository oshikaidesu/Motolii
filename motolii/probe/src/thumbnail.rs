use base64::Engine as _;

/// サムネイルの一辺の上限。DOMへdata URIで埋めるので原寸は載せない。
const MAX_EDGE: u32 = 160;

/// 画像ファイルを縮小してPNGのdata URIにする。読めない物は `None`。
pub fn image_data_uri(path: &str) -> Option<String> {
    let image = image::ImageReader::open(path).ok()?.decode().ok()?;
    encode(image.thumbnail(MAX_EDGE, MAX_EDGE))
}

fn encode(image: image::DynamicImage) -> Option<String> {
    let mut png = std::io::Cursor::new(Vec::new());
    image.write_to(&mut png, image::ImageFormat::Png).ok()?;
    Some(encode_png_bytes(png.into_inner()))
}

fn encode_png_bytes(png: Vec<u8>) -> String {
    let body = base64::engine::general_purpose::STANDARD.encode(png);
    format!("data:image/png;base64,{body}")
}

/// 動画の最初のフレームを縮小してPNGのdata URIにする。`ffmpeg-sidecar` に頼る。
pub fn video_data_uri(path: &str) -> Option<String> {
    use std::io::Read as _;

    let scale = format!("scale={MAX_EDGE}:{MAX_EDGE}:force_original_aspect_ratio=decrease");
    let mut child = ffmpeg_sidecar::command::FfmpegCommand::new()
        .input(path)
        .args(["-vframes", "1", "-vf", &scale, "-f", "image2pipe", "-vcodec", "png"])
        .pipe_stdout()
        .spawn()
        .ok()?;
    let mut png = Vec::new();
    child.take_stdout()?.read_to_end(&mut png).ok()?;
    let status = child.wait().ok()?;
    if !status.success() || png.is_empty() {
        return None;
    }
    Some(encode_png_bytes(png))
}

#[cfg(test)]
mod real_files {
    use super::video_data_uri;

    fn testdata() -> Option<std::path::PathBuf> {
        std::env::var_os("MOTOLII_TESTDATA").map(std::path::PathBuf::from)
    }

    #[test]
    fn video_data_uri_reads_first_frame_of_sample_mp4() {
        let Some(dir) = testdata() else { return };
        let path = dir.join("sample.mp4");
        assert!(path.exists(), "sample.mp4 が無い");
        let uri = video_data_uri(path.to_str().unwrap());
        assert!(uri.as_deref().is_some_and(|u| u.starts_with("data:image/png;base64,")));
    }
}
