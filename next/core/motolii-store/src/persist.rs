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
    pub fn flattened(&self) -> Result<Self, StoreError> {
        let view = self.view();
        let mut out = Self::new();
        let mut intents = Vec::new();

        if let Some(composition) = view.composition()? {
            intents.push(Intent::SetComposition(composition));
        }
        for layer in view.layers() {
            let Some(meta) = view.meta(layer)? else {
                continue;
            };
            intents.push(Intent::AddLayer(layer));
            intents.push(Intent::SetMeta { layer, meta });
            // property の一覧は store に聞く(別の台帳を持たない)。
            for property in view.properties(layer) {
                if let Some(track) = view.track(layer, &property)? {
                    intents.push(Intent::SetTrack {
                        layer,
                        property,
                        track,
                    });
                }
            }
        }

        out.apply_all(intents)?;
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
    pub fn load(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let file = std::fs::File::open(path.as_ref()).map_err(|e| StoreError::Io(e.to_string()))?;
        let decoder = re_log_encoding::rrd::DecoderApp::decode_eager(std::io::BufReader::new(file))
            .map_err(|e| StoreError::Io(e.to_string()))?;

        let mut out = Self::new();
        for message in decoder {
            let message = message.map_err(|e| StoreError::Io(e.to_string()))?;
            out.db
                .add_log_msg(&message)
                .map_err(|e| StoreError::Ingest(e.to_string()))?;
        }
        out.rebuild_head_from_store();
        Ok(out)
    }
}
