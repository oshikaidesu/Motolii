use base64::Engine as _;

const MAX_EDGE: u32 = 160;

/// 一度作った札は取っておく。棚は描き直すたびに全部の札を要求するので、
/// 毎回decodeすると素材が増えるほど窓が重くなる。
static MADE: std::sync::Mutex<Option<std::collections::HashMap<String, Option<String>>>> =
    std::sync::Mutex::new(None);

fn remembered(path: &str, make: impl FnOnce() -> Option<String>) -> Option<String> {
    let mut made = MADE.lock().unwrap_or_else(|e| e.into_inner());
    let map = made.get_or_insert_with(std::collections::HashMap::new);
    if let Some(hit) = map.get(path) {
        return hit.clone();
    }
    let fresh = make();
    map.insert(path.to_string(), fresh.clone());
    fresh
}

pub(super) fn image_data_uri(path: &str) -> Option<String> {
    remembered(path, || {
        let image = image::ImageReader::open(path).ok()?.decode().ok()?;
        encode(image.thumbnail(MAX_EDGE, MAX_EDGE))
    })
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

pub(super) fn video_data_uri(path: &str) -> Option<String> {
    remembered(path, || video_frame(path))
}

fn video_frame(path: &str) -> Option<String> {
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
