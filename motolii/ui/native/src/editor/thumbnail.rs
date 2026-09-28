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

/// 音の札 = 波形そのもの: 素材の全長を等分した [min, max] の列(-1..1)。Timeline と同じ peak の持ち主から読む。
/// 札と同じく一度作ったら取っておく。長い素材(10 分超)は作らない(棚は波の形が読めれば足り、復号は重い)。
static PEAKS: std::sync::Mutex<Option<std::collections::HashMap<String, Option<Vec<[f32; 2]>>>>> =
    std::sync::Mutex::new(None);

pub(crate) const PEAK_COLUMNS: usize = 96;

pub(crate) fn audio_peaks(path: &str) -> Option<Vec<[f32; 2]>> {
    let mut known = PEAKS.lock().unwrap_or_else(|e| e.into_inner());
    let map = known.get_or_insert_with(std::collections::HashMap::new);
    if let Some(hit) = map.get(path) {
        return hit.clone();
    }
    let fresh = make_peaks(path);
    map.insert(path.to_string(), fresh.clone());
    fresh
}

fn make_peaks(path: &str) -> Option<Vec<[f32; 2]>> {
    let seconds = crate::render::media::probe_container(path).ok()?.duration?.as_seconds_f64();
    if !(seconds > 0.0 && seconds <= 600.0) {
        return None;
    }
    let pcm = crate::render::audio::decode_file(path).ok()?;
    let peaks = crate::render::audio::WaveformPeaks::from_pcm(&pcm).ok()?;
    let columns = peaks.columns(0.0, seconds, PEAK_COLUMNS as f64 / seconds)?;
    if columns.is_empty() {
        return None;
    }
    // the pyramid's level gives about PEAK_COLUMNS columns; fold them into exactly that many
    let mut out = vec![[0.0f32, 0.0f32]; PEAK_COLUMNS];
    for c in &columns {
        let i = ((c.at_sec / seconds) * PEAK_COLUMNS as f64).floor().clamp(0.0, (PEAK_COLUMNS - 1) as f64) as usize;
        out[i][0] = out[i][0].min(c.min.clamp(-1.0, 1.0));
        out[i][1] = out[i][1].max(c.max.clamp(-1.0, 1.0));
    }
    Some(out)
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

#[cfg(test)]
mod peak_tests {
    /// A sound's shelf face is its own envelope: loud where it is loud, quiet where it is quiet.
    #[test]
    fn audio_peaks_follow_the_envelope() {
        let dir = std::env::temp_dir().join(format!("motolii-peaks-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("half.wav");
        // 1 s of silence, then 1 s of a full-scale tone
        let ok = std::process::Command::new(crate::render::media::ffmpeg_bin())
            .args(["-v", "error", "-y", "-f", "lavfi", "-i", "aevalsrc=if(lt(t\\,1)\\,0\\,0.9*sin(2*PI*440*t)):s=48000:d=2"])
            .arg(&path)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        assert!(ok, "ffmpeg makes the test sound");
        let peaks = super::audio_peaks(path.to_str().unwrap()).expect("peaks for a 2 s file");
        assert_eq!(peaks.len(), super::PEAK_COLUMNS);
        let span = |c: &[f32; 2]| c[1] - c[0];
        let (first, last) = peaks.split_at(super::PEAK_COLUMNS / 2);
        assert!(first[..first.len() - 2].iter().all(|c| span(c) < 0.05), "the silent half is flat: {first:?}");
        assert!(last[2..].iter().all(|c| span(c) > 1.2), "the loud half fills the face: {last:?}");
        let _ = std::fs::remove_dir_all(dir);
    }
}
