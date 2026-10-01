//! Freeze の cache: 層の「投影の前」の絵(効果の列の出口、乗算済み線形の half float)を、層の時刻ごとに
//! 書類の隣へ置く。法は docs/freeze-and-flatten.md。cache は使い捨てで書類の真実ではない。
//!
//! 置き場: `<root>/<layer id>/<layer frame>.rgba16f` + `.json`(寸法・余白・枠)。
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::doc::store::LayerId;
use crate::render::compositor::effects::vism::ImageFrame;
use crate::render::compositor::GpuTexture2D;

/// 1 コマぶんの凍った絵(GPU に上げた物)と、板として置くのに要る寸法。
#[derive(Clone)]
pub(crate) struct FrozenFrame {
    pub(crate) texture: GpuTexture2D,
    pub(crate) natural: [f32; 2],
    pub(crate) padding: u32,
    pub(crate) frame: Option<ImageFrame>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Meta {
    width: u32,
    height: u32,
    natural: [f32; 2],
    padding: u32,
    frame: Option<(([f32; 2], [f32; 2]), [u32; 2])>,
}

#[derive(Default)]
pub(crate) struct FrozenStore {
    /// 書類の隣の dir。無ければ disk には書かない(GPU の中だけ)。
    pub(crate) root: Option<PathBuf>,
    /// 層 → 層の frame → 上げた絵。disk から戻した物もここに入る。
    frames: HashMap<LayerId, HashMap<i64, FrozenFrame>>,
    /// disk に無いと判った(層, frame)。毎フレーム stat を打たない。
    missing: HashMap<LayerId, std::collections::HashSet<i64>>,
    /// 上げた絵の上限(層をまたいだ合計)。越えたら古い順に GPU から降ろす(disk には残る)。
    resident: std::collections::VecDeque<(LayerId, i64)>,
}

/// GPU に置いておく凍ったコマの上限(1080p の half float で 1 枚 16 MB。240 枚 ≈ 4 GB)。
const RESIDENT_MAX: usize = 240;

impl FrozenStore {
    fn dir(&self, layer: LayerId) -> Option<PathBuf> { self.root.as_ref().map(|r| r.join(layer.0.to_string())) }
    fn paths(&self, layer: LayerId, frame: i64) -> Option<(PathBuf, PathBuf)> {
        let dir = self.dir(layer)?;
        Some((dir.join(format!("{frame}.rgba16f")), dir.join(format!("{frame}.json"))))
    }

    /// GPU にある物。
    pub(crate) fn get(&self, layer: LayerId, frame: i64) -> Option<&FrozenFrame> {
        self.frames.get(&layer)?.get(&frame)
    }

    pub(crate) fn has_on_disk(&mut self, layer: LayerId, frame: i64) -> bool {
        if self.get(layer, frame).is_some() { return true; }
        if self.missing.get(&layer).is_some_and(|m| m.contains(&frame)) { return false; }
        let present = self.paths(layer, frame).is_some_and(|(bytes, meta)| bytes.exists() && meta.exists());
        if !present { self.missing.entry(layer).or_default().insert(frame); }
        present
    }

    /// 1 コマを覚える(GPU)。disk があれば書く。
    pub(crate) fn remember(&mut self, layer: LayerId, frame: i64, picture: FrozenFrame, bytes: Option<&[u8]>) -> std::io::Result<()> {
        if let (Some((bytes_path, meta_path)), Some(bytes)) = (self.paths(layer, frame), bytes) {
            std::fs::create_dir_all(bytes_path.parent().expect("dir"))?;
            let [w, h] = picture.texture.width_height();
            let meta = Meta { width: w, height: h, natural: picture.natural, padding: picture.padding, frame: picture.frame.map(|f| ((f.size, f.origin), f.pixels)) };
            std::fs::write(&bytes_path, bytes)?;
            std::fs::write(&meta_path, serde_json::to_vec(&meta)?)?;
        }
        self.missing.entry(layer).or_default().remove(&frame);
        self.insert_resident(layer, frame, picture);
        Ok(())
    }

    fn insert_resident(&mut self, layer: LayerId, frame: i64, picture: FrozenFrame) {
        self.frames.entry(layer).or_default().insert(frame, picture);
        self.resident.push_back((layer, frame));
        while self.resident.len() > RESIDENT_MAX {
            if let Some((old_layer, old_frame)) = self.resident.pop_front() {
                if let Some(per_layer) = self.frames.get_mut(&old_layer) { per_layer.remove(&old_frame); }
            }
        }
    }

    /// disk から戻す(GPU へ上げる)。無ければ None。
    pub(crate) fn load(&mut self, layer: LayerId, frame: i64, upload: &mut dyn FnMut(Vec<u8>, u32, u32) -> Option<GpuTexture2D>) -> Option<FrozenFrame> {
        if let Some(have) = self.get(layer, frame) { return Some(have.clone()); }
        if !self.has_on_disk(layer, frame) { return None; }
        let (bytes_path, meta_path) = self.paths(layer, frame)?;
        let meta: Meta = serde_json::from_slice(&std::fs::read(meta_path).ok()?).ok()?;
        let bytes = std::fs::read(bytes_path).ok()?;
        if bytes.len() != (meta.width * meta.height * 8) as usize { return None; }
        let texture = upload(bytes, meta.width, meta.height)?;
        let picture = FrozenFrame { texture, natural: meta.natural, padding: meta.padding, frame: meta.frame.map(|((size, origin), pixels)| ImageFrame { size, origin, pixels }) };
        self.insert_resident(layer, frame, picture.clone());
        Some(picture)
    }

    pub(crate) fn forget_missing(&mut self) { self.missing.clear(); }

    /// 層の cache を全部捨てる(Unfreeze)。disk の dir も消す。
    pub(crate) fn forget(&mut self, layer: LayerId) {
        self.frames.remove(&layer);
        self.missing.remove(&layer);
        self.resident.retain(|(l, _)| *l != layer);
        if let Some(dir) = self.dir(layer) { let _ = std::fs::remove_dir_all(dir); }
    }

    /// 凍った層の、GPU にあるコマの数(試験と status 用)。
    pub(crate) fn resident_count(&self, layer: LayerId) -> usize { self.frames.get(&layer).map_or(0, |m| m.len()) }

    pub(crate) fn frames_on_disk(&self, layer: LayerId) -> usize {
        self.dir(layer).and_then(|d| std::fs::read_dir(d).ok()).map_or(0, |rd| rd.filter_map(Result::ok).filter(|e| e.path().extension().is_some_and(|x| x == "rgba16f")).count())
    }

    pub(crate) fn root_for_document(path: Option<&Path>) -> Option<PathBuf> {
        let path = path?;
        let stem = path.file_stem()?.to_string_lossy().into_owned();
        Some(path.with_file_name(format!("{stem}.motolii-cache")))
    }
}
