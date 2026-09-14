//! 解析の入力 — 描いた絵から host が解いた値(Blob の塊など)を、resolve が読むための口(2026-09-14 利用者裁定)。
//! 書類には入れない再生成可能なキャッシュ(simulation-model.md §3.3「文書とキャッシュの分離」)。書類に入るのはレシピ
//! (効果と取っ手)だけで、host がコマごとに解いてここへ置き、resolve は入力として読むだけ(純関数のまま)。
//! 同じ口に、後でデータモッシュの動きの場なども載る。

use std::collections::HashMap;

use crate::doc::core::RationalTime;
use crate::doc::store::{EffectId, LayerId};

/// 1 つの塊。位置と大きさは comp の px(元の層の置き場所を通した後)。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlobMark {
    pub id: u32,
    pub center: [f32; 2],
    pub size: [f32; 2],
    /// 同じ ID で続いたコマ数(0 は生まれたコマ)。
    pub age: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnalysisInputs {
    blobs: HashMap<(LayerId, EffectId, i64, i64), Vec<BlobMark>>,
    /// 素材ファイルの元の寸法(幅・高さ・奥行き、素材座標)。画・動画は奥行き 0、網・点群は bounds。並べる法の箱。
    extents: HashMap<String, [f32; 3]>,
}

impl AnalysisInputs {
    pub fn set_blobs(&mut self, layer: LayerId, effect: EffectId, t: RationalTime, marks: Vec<BlobMark>) {
        self.blobs.insert((layer, effect, t.num(), t.den()), marks);
    }

    pub fn blobs(&self, layer: LayerId, effect: EffectId, t: RationalTime) -> Option<&[BlobMark]> {
        self.blobs.get(&(layer, effect, t.num(), t.den())).map(Vec::as_slice)
    }

    pub fn set_extent(&mut self, path: &str, extent: [f32; 3]) {
        self.extents.insert(path.to_owned(), extent);
    }

    pub fn extent(&self, path: &str) -> Option<[f32; 3]> {
        self.extents.get(path).copied()
    }

    pub fn is_empty(&self) -> bool {
        self.blobs.is_empty() && self.extents.is_empty()
    }
}
