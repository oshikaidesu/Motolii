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
    let body = base64::engine::general_purpose::STANDARD.encode(png.into_inner());
    Some(format!("data:image/png;base64,{body}"))
}
