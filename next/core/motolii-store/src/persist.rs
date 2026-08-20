//! 保存と読込。
//!
//! **形式は上流の `.rrd` をそのまま使う**(自前形式を発明しない、軸4)。
//! store の中身がそのまま file になり、読むと store が戻る。
//!
//! **保存する時は履歴を畳む**。store は全 edit 刻みを持っているので、そのまま書くと
//! project file が編集回数に比例して伸びる(R0-1 実測で 1000編集 × 300打点 = 18.8MB)。
//! 保存は「今見えている状態を、新しい Document として書く」形にしてある。
//! セッションを跨いだ undo は捨てる — 普通の編集ソフトと同じ挙動。
//!
//! **危険**: `.rrd` は rerun の形式なので、fork の rev を上げると古い project が
//! 読めなくなりうる。上流は互換性を保つ努力をしているが保証ではないので、
//! rev を上げる時は**この試験(往復)を必ず回す**。

use std::path::Path;

use crate::{Document, Intent, StoreError};

impl Document {
    /// 今見えている状態だけを持つ新しい Document を作る(履歴を畳む)。
    ///
    /// **component を名前で列挙しない**(裁定108(a) の構造修正)。以前はここに
    /// `meta`/`masks`/`markers`/`composition`/property track を1つずつ名指しした
    /// コピー処理が並んでおり、**新しい component を1つ足すたびにここを直す**必要が
    /// あった — 直し忘れると保存から黙って消える。今は
    /// [`crate::StoreView::track_json_components`] が「この entity が今持っている
    /// component 全部」を store に聞いて返すので、`attrs`/`effects`/`shapes`/`text`
    /// のような後発の component も1行も足さずに運ばれる(裁定57)。
    ///
    /// `AddLayer`(存在)だけは別扱い — `TrackJson` ではなく `LayerPresent`(bool)で、
    /// かつ「今 present な layer の集合」は `view.layers()` が既に tombstone を
    /// 弾いた形で返すため。
    pub fn flattened(&self) -> Result<Self, StoreError> {
        let view = self.view();
        let mut out = Self::new();
        // 履歴を1 edit 刻みへ畳む(裁定56)。全コピーを同じ `at` で書く。
        let at = 1;

        for (component, json) in view.track_json_components(&Document::composition_path()) {
            out.copy_track_json(
                Document::composition_path(),
                component,
                "motolii.archetypes.Composition",
                json,
                at,
            )?;
        }

        for layer in view.layers() {
            out.write(Intent::AddLayer(layer), at)?;
            for (component, json) in view.track_json_components(&layer.entity_path()) {
                out.copy_track_json(layer.entity_path(), component, "motolii.archetypes.Layer", json, at)?;
            }
        }

        // 開いた直後は「編集していない」ので戻せない。
        out.mark_undo_floor();
        Ok(out)
    }

    /// 保存。履歴は畳む。
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), StoreError> {
        let flat = self.flattened()?;
        let file = std::fs::File::create(path.as_ref())
            .map_err(|e| StoreError::Io(e.to_string()))?;
        let mut encoder = re_log_encoding::rrd::Encoder::new_eager(
            re_build_info::CrateVersion::LOCAL,
            re_log_encoding::rrd::EncodingOptions::PROTOBUF_UNCOMPRESSED,
            std::io::BufWriter::new(file),
        )
        .map_err(|e| StoreError::Io(e.to_string()))?;

        for message in flat.db.to_messages(None) {
            let message = message.map_err(|e| StoreError::Io(e.to_string()))?;
            encoder
                .append(&message)
                .map_err(|e| StoreError::Io(e.to_string()))?;
        }
        encoder.finish().map_err(|e| StoreError::Io(e.to_string()))?;
        Ok(())
    }

    /// 読込。開いた直後は**戻せない**(`mark_undo_floor`)。
    ///
    /// **store の同一性は file が持つ**。`Document::new()` で新しい `StoreId` を作ってから
    /// file のメッセージを流し込むと、store 自身の id とメッセージの id が食い違う。
    /// upstream は `debug_assert` で叩くだけなので release では黙って通り、
    /// **保存した file と store の同一性がずれたまま動く**(2026-08-20、shape-1 レーンが
    /// debug ビルドで踏んで発覚)。
    pub fn load(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let file = std::fs::File::open(path.as_ref()).map_err(|e| StoreError::Io(e.to_string()))?;
        let decoder = re_log_encoding::rrd::DecoderApp::decode_eager(std::io::BufReader::new(file))
            .map_err(|e| StoreError::Io(e.to_string()))?;

        let mut out: Option<Self> = None;
        for message in decoder {
            let message = message.map_err(|e| StoreError::Io(e.to_string()))?;
            let doc = match &mut out {
                Some(doc) => doc,
                None => {
                    // 最初のメッセージが store の同一性を決める。
                    out = Some(Self::with_store_id(message.store_id().clone()));
                    out.as_mut().expect("直前に入れた")
                }
            };
            doc.db
                .add_log_msg(&message)
                .map_err(|e| StoreError::Ingest(e.to_string()))?;
        }

        let mut out = out.ok_or_else(|| StoreError::Io("空の file(メッセージが1つも無い)".into()))?;
        out.rebuild_head_from_store();
        out.mark_undo_floor();
        Ok(out)
    }
}
