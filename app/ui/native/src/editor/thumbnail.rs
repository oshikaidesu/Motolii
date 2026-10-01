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

/// 取り込みの糸で先に札を作っておく。描画の最中に decode / ffmpeg を回さない(M6)。
pub(crate) fn warm(path: &str, video: bool) {
    if video {
        let _ = video_data_uri(path);
    } else {
        let _ = image_data_uri(path);
    }
}

pub(crate) fn image_data_uri(path: &str) -> Option<String> {
    remembered(path, || {
        // Stage と同じ decode(ICC 適用)。札と絵で色が違うと素材を疑う。
        let (rgba, width, height) = crate::render::media::decode_still_srgb(path).ok()?;
        let image = image::DynamicImage::ImageRgba8(image::RgbaImage::from_raw(width, height, rgba)?);
        encode(image.thumbnail(MAX_EDGE, MAX_EDGE))
    })
}

fn encode(image: image::DynamicImage) -> Option<String> {
    let mut png = std::io::Cursor::new(Vec::new());
    image.write_to(&mut png, image::ImageFormat::Png).ok()?;
    encode_png_bytes(png.into_inner())
}

/// 絵として読める PNG だけを data URI にする。壊れた bytes を描画へ渡すと、
/// renderer が「空の image」で落ちる(実測: 起動時に vello が panic)。
fn encode_png_bytes(png: Vec<u8>) -> Option<String> {
    let ok = image::load_from_memory_with_format(&png, image::ImageFormat::Png)
        .map(|img| img.width() > 0 && img.height() > 0)
        .unwrap_or(false);
    if !ok {
        return None;
    }
    let body = base64::engine::general_purpose::STANDARD.encode(png);
    Some(format!("data:image/png;base64,{body}"))
}

pub(crate) fn video_data_uri(path: &str) -> Option<String> {
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
    encode_png_bytes(png)
}

/// 素材の事実: 寸法・fps・尺・音の標本化周波数と ch。札と同じく一度読んだら取っておく。
/// 画は頭だけ読む(decode しない)。動画と音は ffprobe。読めない物は空のまま。
static FACTS: std::sync::Mutex<Option<std::collections::HashMap<String, Option<serde_json::Value>>>> =
    std::sync::Mutex::new(None);

pub(crate) fn facts(path: &str, mime: &str) -> Option<serde_json::Value> {
    let mut known = FACTS.lock().unwrap_or_else(|e| e.into_inner());
    let map = known.get_or_insert_with(std::collections::HashMap::new);
    if let Some(hit) = map.get(path) {
        return hit.clone();
    }
    let fresh = make_facts(path, mime);
    map.insert(path.to_string(), fresh.clone());
    fresh
}

fn make_facts(path: &str, mime: &str) -> Option<serde_json::Value> {
    use serde_json::json;
    if mime.starts_with("image/") {
        let (width, height) = image::ImageReader::open(path).ok()?.into_dimensions().ok()?;
        return Some(json!({"width": width, "height": height}));
    }
    if mime.starts_with("video/") {
        let info = crate::render::media::probe(path).ok()?;
        return Some(json!({
            "width": info.width,
            "height": info.height,
            "fps": info.fps.as_f64(),
            "seconds": info.duration.map(|d| d.as_seconds_f64()),
        }));
    }
    if mime.starts_with("audio/") {
        let container = crate::render::media::probe_container(path).ok()?;
        let stream = container.audio_streams.first()?;
        return Some(json!({
            "sampleRate": stream.sample_rate,
            "channels": stream.channels,
            "seconds": container.duration.map(|d| d.as_seconds_f64()),
        }));
    }
    None
}
