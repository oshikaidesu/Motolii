

use motolii_store::{
    AssetDraft, AssetId, Intent, LayerAttrsPatch, LayerId,
    LayerMeta, LayerSource, LayerTiming,
    SourceFingerprintV1,
};

use crate::{
    browser_pane, Shell,
};

impl Shell {
    /// 落ちてきた path を素材として受ける。
    ///
    /// **開けない物は理由つきで飛ばす**(M2)。黙って消すと利用者は
    /// 「落としたのに何も起きない」としか分からない。
    ///
    /// 裁定162(B1、bin-first の下地): 各 path は**まず台帳へ記帳**
    /// (`Intent::AdmitAsset`)し、その上で従来どおり layer として配置する。
    /// 記帳と配置は別の関心事 — 記帳は「fingerprint が計算できたか」だけを見て
    /// 判定し、配置できるかどうか(`motolii_media::probe` が成功するか)を
    /// 問わない。junk file(probe が失敗する物)でも fingerprint さえ読めれば
    /// 台帳には載る(bin-first: 取り込みと配置は別の判断)。同一ファイルの
    /// 再 drop は `AssetTable::admit` の content_hash 重複統合にそのまま乗る
    /// (shell 側で先回りの dedupe はしない、EXACT TARGET #3)。
    pub(crate) fn admit(&mut self, paths: Vec<std::path::PathBuf>) {
        let mut intents = Vec::new();
        let mut rejected = Vec::new();
        let mut admission_skipped = Vec::new();
        let mut next = self.next_layer_id();

        let comp_duration = self.comp_duration();
        let start = self.session.playhead;
        let _ = start;

        // 第6波(B08 取り込み UX 結線): 記帳前の台帳 id 集合を控え、記帳後との
        // 差分で「今回新規に載った id」を出す(`Intent::AdmitAsset` 自体は
        // 割り当てた id を呼び手へ返さないので、この差分が唯一の道)。
        let before_admit: std::collections::HashSet<AssetId> =
            self.assets().into_iter().map(|item| item.id).collect();

        for path in paths {
            let text = path.to_string_lossy().into_owned();
            let file_name = || {
                path.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| text.clone())
            };

            // **記帳**(台帳、裁定162)。fingerprint 計算(ファイル IO)が失敗
            // したら記帳だけスキップする — 配置(下の probe)は独立に続行する。
            match Self::fingerprint_source(&path) {
                Ok(fingerprint) => {
                    let draft = AssetDraft::from_probed_source(
                        Self::guess_asset_type(&path),
                        &fingerprint,
                        &path,
                        None,
                    );
                    intents.push(Intent::AdmitAsset { draft });
                }
                Err(error) => {
                    admission_skipped.push(format!("{}: {error}", file_name()));
                }
            }

            // **配置**(従来どおり)。
            match motolii_media::probe(&path) {
                Ok(info) => {
                    let id = LayerId(next);
                    next += 1;
                    intents.push(Intent::AddLayer(id));
                    intents.push(Intent::SetMeta {
                        layer: id,
                        meta: LayerMeta {
                            source: LayerSource::Media {
                                path: text,
                                fingerprint: None,
                            },
                            order: id.0 as i16,
                            timing: LayerTiming::place(
                                self.session.playhead,
                                info.nb_frames,
                                comp_duration,
                            ),
                        },
                    });
                    // 差し色の自動割当(`Message::AddLayer` と同じ決定論、
                    // `label_color_for_new_layer` 参照)。
                    intents.push(Intent::SetAttrs {
                        layer: id,
                        patch: LayerAttrsPatch {
                            label_color: Some(Some(Self::label_color_for_new_layer(id))),
                            ..Default::default()
                        },
                    });
                }
                Err(error) => {
                    rejected.push(format!("{}: {error}", file_name()));
                }
            }
        }

        // 落とした分は**まとめて1 undo**(1操作 = 1 undo)。台帳記帳(AdmitAsset)も
        // 同じ batch に同居させる — 呼び手(`Message::AdmitPaths`/`FlushDrops`)が
        // 渡した path 列ぜんぶで1 undo という既存の粒をそのまま保つ(1 path = 1
        // undo ではない、`admit` の doc 冒頭参照)。
        if !intents.is_empty() {
            if let Err(error) = self.doc.apply_all(intents) {
                rejected.push(format!("置けなかった: {error}"));
            }
        }
        // 記帳後との差分 = 今回新規に載った id(`AdmitAsset` が失敗した分は
        // 台帳に載っていないので自動的に含まれない)。`RecentlyAdmitted` は
        // カード選択かタブ切替で消灯する表示専用状態(`browser_pane::state`
        // doc 参照)— 空なら publish しない(no-op で pane 側の既存点灯を
        // 消さない)。
        let admitted: Vec<AssetId> = self
            .assets()
            .into_iter()
            .map(|item| item.id)
            .filter(|id| !before_admit.contains(id))
            .collect();
        if !admitted.is_empty() {
            self.browser.update(browser_pane::Message::RecentlyAdmitted(admitted));
        }
        let mut notices = Vec::new();
        if !rejected.is_empty() {
            notices.push(format!(
                "受け取れない素材 {}件 — {}",
                rejected.len(),
                rejected.join(" / ")
            ));
        }
        if !admission_skipped.is_empty() {
            notices.push(format!(
                "台帳への記帳をスキップ {}件 — {}",
                admission_skipped.len(),
                admission_skipped.join(" / ")
            ));
        }
        if !notices.is_empty() {
            self.status = Some(notices.join(" / "));
        }
    }

    /// `Intent::AdmitAsset` の draft を組むための fingerprint 計算(ファイル IO)。
    /// `motolii_media::probe`(ffprobe サイドカー)とは独立 — 記帳は「読めるか」
    /// だけを見る(EXACT TARGET #2)。
    pub(crate) fn fingerprint_source(
        path: &std::path::Path,
    ) -> Result<SourceFingerprintV1, motolii_store::SourceFingerprintError> {
        let file = std::fs::File::open(path)?;
        SourceFingerprintV1::from_reader(file)
    }

    /// 台帳の `asset_type`(opaque 文字列)を拡張子から粗く推定する。**種別判定の
    /// 精度はこの切片(B1)の非目標** — rail/filter(B2)以降が正確な種別判定
    /// (意味起草タスク#14 の空席)を持つまでの暫定値。
    pub(crate) fn guess_asset_type(path: &std::path::Path) -> String {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        match ext.as_str() {
            "mp4" | "mov" | "mkv" | "webm" | "avi" | "m4v" => format!("video/{ext}"),
            "jpg" | "jpeg" => "image/jpeg".to_owned(),
            "png" | "gif" | "webp" | "bmp" | "svg" => format!("image/{ext}"),
            "wav" | "mp3" | "aac" | "flac" | "ogg" | "m4a" => format!("audio/{ext}"),
            "" => "application/octet-stream".to_owned(),
            other => format!("application/{other}"),
        }
    }

}

