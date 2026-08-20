//! owns: 「binary_little_endian PLY を re_renderer の点群として撮影できるか」の実測
//!       (D12「rerunだからこそ3D」— 点群・深度クラウド・3Dメッシュの撮影は GOALS.md で
//!       まだ「未」になっている実データ受入)。
//!
//! rerun 本体には viewer 層(`re_data_loader` 等)に PLY loader があるが、それは egui
//! viewer app にぶら下がっており、この workspace は viewer 層を引かない(裁定3)。
//! だからここは **header を実際にパースして stride を決める最小のバイナリ読み取り**
//! だけを持つ — ASCII PLY・非 little-endian・可変長 property(list 型)は対象外。
//! 利用者の実データ(PCL 出力の `fragment.ply`)が読めれば足りる probe である。

use std::io::Read as _;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum PlyError {
    #[error("ファイルを開けない: {0}")]
    Io(#[from] std::io::Error),
    #[error("PLY header が読めない: {0}")]
    Header(String),
    #[error("この header は対象外(binary_little_endian の vertex を読むだけの最小パーサ): {0}")]
    Unsupported(String),
}

/// PLY の scalar property 型。この probe が対応する分だけ(list 型は非対応)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScalarType {
    F32,
    U8,
    I32,
}

impl ScalarType {
    fn size(self) -> usize {
        match self {
            Self::F32 | Self::I32 => 4,
            Self::U8 => 1,
        }
    }

    fn parse(name: &str) -> Option<Self> {
        match name {
            "float" | "float32" => Some(Self::F32),
            "uchar" | "uint8" | "char" | "int8" => Some(Self::U8),
            "int" | "int32" | "uint" | "uint32" => Some(Self::I32),
            _ => None,
        }
    }
}

struct ElementDesc {
    name: String,
    count: usize,
    properties: Vec<(String, ScalarType)>,
}

/// 読み取った点群。位置は PLY のまま(単位変換なし)、色は RGB をそのまま。
#[derive(Debug)]
pub struct PlyPointCloud {
    pub positions: Vec<[f32; 3]>,
    pub colors: Vec<[u8; 3]>,
}

impl PlyPointCloud {
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    /// bounding box (min, max)。
    pub fn bounds(&self) -> (glam::Vec3, glam::Vec3) {
        let mut min = glam::Vec3::splat(f32::INFINITY);
        let mut max = glam::Vec3::splat(f32::NEG_INFINITY);
        for p in &self.positions {
            let v = glam::Vec3::from(*p);
            min = min.min(v);
            max = max.max(v);
        }
        (min, max)
    }

    /// 全点の重心。
    pub fn centroid(&self) -> glam::Vec3 {
        let sum: glam::Vec3 = self
            .positions
            .iter()
            .fold(glam::Vec3::ZERO, |acc, p| acc + glam::Vec3::from(*p));
        sum / self.positions.len().max(1) as f32
    }

    /// 重心からの最大距離(外接球の半径。カメラ距離を決めるのに使う)。
    pub fn bounding_radius(&self, center: glam::Vec3) -> f32 {
        self.positions
            .iter()
            .map(|p| (glam::Vec3::from(*p) - center).length())
            .fold(0.0f32, f32::max)
    }
}

/// `end_header` 行の直後(バイナリ本体の先頭)を探す。改行は `\n` と `\r\n` の
/// どちらもありうる(fragment.ply は実測 `\r\n` — PLY spec は行区切りの厳密な指定を
/// せず、書き出し側の慣習に幅がある)。
fn find_header_end(bytes: &[u8]) -> Option<usize> {
    const NEEDLE: &[u8] = b"end_header";
    let start = bytes.windows(NEEDLE.len()).position(|w| w == NEEDLE)?;
    let mut i = start + NEEDLE.len();
    if bytes.get(i) == Some(&b'\r') {
        i += 1;
    }
    if bytes.get(i) == Some(&b'\n') {
        i += 1;
    }
    Some(i)
}

/// binary_little_endian PLY を読む。**header を実際にパースして stride/offset を決める**
/// — property の並びや型をこの関数にハードコードしない。
pub fn load_binary_ply(path: &Path) -> Result<PlyPointCloud, PlyError> {
    let mut file = std::fs::File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    parse_binary_ply(&bytes)
}

fn parse_binary_ply(bytes: &[u8]) -> Result<PlyPointCloud, PlyError> {
    let header_end = find_header_end(bytes)
        .ok_or_else(|| PlyError::Header("end_header が見つからない".into()))?;
    let header_text = std::str::from_utf8(&bytes[..header_end])
        .map_err(|e| PlyError::Header(format!("header が UTF-8 でない: {e}")))?;

    let mut lines = header_text.lines();
    let magic = lines.next().unwrap_or_default().trim();
    if magic != "ply" {
        return Err(PlyError::Unsupported(format!(
            "先頭が 'ply' でない: {magic:?}"
        )));
    }

    let mut elements: Vec<ElementDesc> = Vec::new();
    let mut saw_format = false;

    for line in lines {
        let line = line.trim();
        if line.is_empty() || line.starts_with("comment") || line.starts_with("obj_info") {
            continue;
        }
        if line == "end_header" {
            break;
        }
        if let Some(rest) = line.strip_prefix("format ") {
            if !rest.trim_start().starts_with("binary_little_endian") {
                return Err(PlyError::Unsupported(format!(
                    "binary_little_endian 以外の format は対象外: {rest:?}"
                )));
            }
            saw_format = true;
            continue;
        }
        if let Some(rest) = line.strip_prefix("element ") {
            let mut parts = rest.split_whitespace();
            let name = parts
                .next()
                .ok_or_else(|| PlyError::Header(format!("element 行が壊れている: {line:?}")))?;
            let count: usize = parts
                .next()
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| PlyError::Header(format!("element の count が読めない: {line:?}")))?;
            elements.push(ElementDesc {
                name: name.to_string(),
                count,
                properties: Vec::new(),
            });
            continue;
        }
        if let Some(rest) = line.strip_prefix("property ") {
            let element = elements.last_mut().ok_or_else(|| {
                PlyError::Header(format!("property が element より前に出た: {line:?}"))
            })?;
            let mut parts = rest.split_whitespace();
            let ty = parts
                .next()
                .ok_or_else(|| PlyError::Header(format!("property 行が壊れている: {line:?}")))?;
            if ty == "list" {
                return Err(PlyError::Unsupported(format!(
                    "可変長 property(list)は対象外: {line:?}"
                )));
            }
            let scalar = ScalarType::parse(ty)
                .ok_or_else(|| PlyError::Unsupported(format!("未対応の property 型: {ty:?}")))?;
            let name = parts
                .next()
                .ok_or_else(|| PlyError::Header(format!("property 名が読めない: {line:?}")))?;
            element.properties.push((name.to_string(), scalar));
            continue;
        }
        // 未知行は無視(将来の comment 亜種等)。
    }

    if !saw_format {
        return Err(PlyError::Header("format 行が無い".into()));
    }

    let vertex_index = elements
        .iter()
        .position(|e| e.name == "vertex")
        .ok_or_else(|| PlyError::Header("element vertex が無い".into()))?;
    // vertex より前に別 element があると、そこのバイト分だけ本体の開始位置がずれる。
    // この probe が扱う実データ(fragment.ply)は vertex が先頭 element なので、
    // その前提が崩れたら黙って壊れた読みを返さず Unsupported にする。
    if vertex_index != 0 {
        return Err(PlyError::Unsupported(
            "vertex が先頭 element でない構成は対象外(offset 計算を実装していない)".into(),
        ));
    }
    let vertex = &elements[vertex_index];

    let offset_of = |field: &str| -> Result<(usize, ScalarType), PlyError> {
        let mut offset = 0usize;
        for (name, ty) in &vertex.properties {
            if name == field {
                return Ok((offset, *ty));
            }
            offset += ty.size();
        }
        Err(PlyError::Header(format!(
            "vertex に property {field:?} が無い"
        )))
    };
    let stride: usize = vertex.properties.iter().map(|(_, ty)| ty.size()).sum();

    let (x_off, x_ty) = offset_of("x")?;
    let (y_off, _) = offset_of("y")?;
    let (z_off, _) = offset_of("z")?;
    let (r_off, r_ty) = offset_of("red")?;
    let (g_off, _) = offset_of("green")?;
    let (b_off, _) = offset_of("blue")?;
    if x_ty != ScalarType::F32 {
        return Err(PlyError::Unsupported("x/y/z が float でない".into()));
    }
    if r_ty != ScalarType::U8 {
        return Err(PlyError::Unsupported("red/green/blue が uchar でない".into()));
    }

    let body = &bytes[header_end..];
    let needed = stride * vertex.count;
    if body.len() < needed {
        return Err(PlyError::Header(format!(
            "vertex データが header の宣言より短い: 必要 {needed}B, 実際 {}B",
            body.len()
        )));
    }

    let mut positions = Vec::with_capacity(vertex.count);
    let mut colors = Vec::with_capacity(vertex.count);
    for i in 0..vertex.count {
        let row = &body[i * stride..(i + 1) * stride];
        let x = f32::from_le_bytes(row[x_off..x_off + 4].try_into().unwrap());
        let y = f32::from_le_bytes(row[y_off..y_off + 4].try_into().unwrap());
        let z = f32::from_le_bytes(row[z_off..z_off + 4].try_into().unwrap());
        positions.push([x, y, z]);
        colors.push([row[r_off], row[g_off], row[b_off]]);
    }

    Ok(PlyPointCloud { positions, colors })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// vertex 2点だけの最小 binary_little_endian PLY を手で組み立てる。
    /// property の並びは実データ(fragment.ply)と同じにして、offset 計算が
    /// ハードコードでなく header 由来であることを縛る。
    fn synthetic_ply() -> Vec<u8> {
        let header = "ply\n\
format binary_little_endian 1.0\n\
comment PCL generated\n\
element vertex 2\n\
property float x\n\
property float y\n\
property float z\n\
property uchar red\n\
property uchar green\n\
property uchar blue\n\
property float nx\n\
property float ny\n\
property float nz\n\
property float curvature\n\
element camera 1\n\
property float view_px\n\
end_header\n";

        let mut bytes = header.as_bytes().to_vec();
        // vertex 0
        bytes.extend_from_slice(&1.0f32.to_le_bytes());
        bytes.extend_from_slice(&2.0f32.to_le_bytes());
        bytes.extend_from_slice(&3.0f32.to_le_bytes());
        bytes.extend_from_slice(&[10u8, 20u8, 30u8]);
        bytes.extend_from_slice(&0.0f32.to_le_bytes());
        bytes.extend_from_slice(&0.0f32.to_le_bytes());
        bytes.extend_from_slice(&1.0f32.to_le_bytes());
        bytes.extend_from_slice(&0.5f32.to_le_bytes());
        // vertex 1
        bytes.extend_from_slice(&4.0f32.to_le_bytes());
        bytes.extend_from_slice(&5.0f32.to_le_bytes());
        bytes.extend_from_slice(&6.0f32.to_le_bytes());
        bytes.extend_from_slice(&[40u8, 50u8, 60u8]);
        bytes.extend_from_slice(&1.0f32.to_le_bytes());
        bytes.extend_from_slice(&0.0f32.to_le_bytes());
        bytes.extend_from_slice(&0.0f32.to_le_bytes());
        bytes.extend_from_slice(&0.25f32.to_le_bytes());
        // camera element のデータは意図的に書かない — vertex が先頭 element である限り
        // 読まれないことを確かめるのがこの試験の要点でもある。
        bytes
    }

    #[test]
    fn parses_positions_and_colors_from_header_derived_offsets() {
        let cloud = parse_binary_ply(&synthetic_ply()).expect("valid ply");
        assert_eq!(cloud.len(), 2);
        assert_eq!(cloud.positions[0], [1.0, 2.0, 3.0]);
        assert_eq!(cloud.positions[1], [4.0, 5.0, 6.0]);
        assert_eq!(cloud.colors[0], [10, 20, 30]);
        assert_eq!(cloud.colors[1], [40, 50, 60]);
    }

    #[test]
    fn centroid_and_bounds_match_hand_computed_values() {
        let cloud = parse_binary_ply(&synthetic_ply()).expect("valid ply");
        let centroid = cloud.centroid();
        assert!((centroid - glam::vec3(2.5, 3.5, 4.5)).length() < 1e-5);
        let (min, max) = cloud.bounds();
        assert_eq!(min, glam::vec3(1.0, 2.0, 3.0));
        assert_eq!(max, glam::vec3(4.0, 5.0, 6.0));
    }

    #[test]
    fn rejects_ascii_format() {
        let header = "ply\nformat ascii 1.0\nelement vertex 1\nproperty float x\nend_header\n";
        let err = parse_binary_ply(header.as_bytes()).unwrap_err();
        assert!(matches!(err, PlyError::Unsupported(_)));
    }

    #[test]
    fn rejects_list_properties() {
        let header = "ply\nformat binary_little_endian 1.0\n\
element vertex 1\nproperty float x\nproperty list uchar int vertex_indices\nend_header\n";
        let err = parse_binary_ply(header.as_bytes()).unwrap_err();
        assert!(matches!(err, PlyError::Unsupported(_)));
    }

    #[test]
    fn rejects_vertex_not_first_element() {
        let header = "ply\nformat binary_little_endian 1.0\n\
element camera 1\nproperty float view_px\nelement vertex 1\nproperty float x\nend_header\n";
        let err = parse_binary_ply(header.as_bytes()).unwrap_err();
        assert!(matches!(err, PlyError::Unsupported(_)));
    }
}
