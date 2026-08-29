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

use std::path::{Path, PathBuf};

use crate::components::{archetype_composition, archetype_layer};
use crate::{Document, Intent, Revision, StoreError};

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

        for (component, json) in view.track_json_components(&Document::composition_path())? {
            out.copy_track_json(
                Document::composition_path(),
                component,
                archetype_composition(),
                json,
                at,
            )?;
        }

        for layer in view.layers() {
            out.write(Intent::AddLayer(layer), at)?;
            for (component, json) in view.track_json_components(&layer.entity_path())? {
                out.copy_track_json(layer.entity_path(), component, archetype_layer(), json, at)?;
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
        Self::encode_flattened_to(&flat, file)
    }

    /// `save`/`save_atomic` 共通の符号化本体(履歴を畳んだ後の書き出しだけを持つ)。
    /// 二重管理(手直し漏れ)を避けるための共通化 — 意味は増やしていない。
    fn encode_flattened_to(flat: &Self, file: std::fs::File) -> Result<(), StoreError> {
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

    /// atomic 保存: `path` と同じディレクトリに `.tmp` を書いてから `rename` で
    /// 差し替える。**書き込みが失敗しても既存の `path` は無傷のまま** —
    /// `rename` は書き込み完了後にしか起きない。自動保存(下記 [`Self::auto_save`])が
    /// この経路を使う。明示 Save(`save`)は据え置き(既存呼び手の挙動を変えない)。
    fn save_atomic(&self, path: &Path) -> Result<(), StoreError> {
        let flat = self.flattened()?;
        let dir = path.parent().filter(|p| !p.as_os_str().is_empty());
        let dir = dir.unwrap_or_else(|| Path::new("."));
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| StoreError::Io("保存先のファイル名を取得できない".into()))?;
        // 同じディレクトリに置く(`rename` が同一ファイルシステム内でしか
        // atomic を保証しないため)。先頭 `.` で隠しファイル化しておく。
        let tmp_path = dir.join(format!(".{file_name}.tmp"));

        let file = std::fs::File::create(&tmp_path).map_err(|e| StoreError::Io(e.to_string()))?;
        if let Err(err) = Self::encode_flattened_to(&flat, file) {
            // 書きかけの tmp は片付ける。消せなくても致命ではない(次回上書きされる)。
            let _ = std::fs::remove_file(&tmp_path);
            return Err(err);
        }

        std::fs::rename(&tmp_path, path).map_err(|e| StoreError::Io(e.to_string()))
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

    /// 自動保存(AE `Auto-Save` 先例)。**タイマーはここでは引かない** — 何秒おきに
    /// 呼ぶかは shell の subscription 側の責務で、この関数は「今この瞬間に自動保存
    /// してよいか」を判定して書くだけ。呼び手は tick のたびに呼べばよい。
    ///
    /// # 保存先規約(AE 型)
    ///
    /// `project_path` が `/dir/Foo.motolii` なら、隣に `/dir/Foo auto-save/` を作り、
    /// その中へ `Foo.autosave-{seq}.motolii` を置く(`seq` は単調増加、ディレクトリの
    /// 中身だけから毎回再計算するので**プロセスをまたいでも別の状態を持たない**)。
    /// 保持世代数(`config.generations`)を超えた古い `seq` は書き込み成功後に削除する
    /// (削除できなくても致命ではない — 次回また削除を試みるだけ)。
    ///
    /// mtime ではなく**ファイル名に埋めた seq でローテーションを判定する** —
    /// ファイルシステムの mtime 分解能(粗い環境だと1秒刻み)に依存すると、短い間隔で
    /// 連続保存した時に「どれが最古か」を取り違える。
    ///
    /// # dirty 判定
    ///
    /// `since` は「最後に(明示 Save か自動保存で)書いた時点の [`Revision`]」— 呼び手
    /// (shell)が保持し、書けたら都度その時の `document.revision()` へ更新すること。
    /// `self.revision() == *since` なら**何もせず `Ok(None)`** を返す(そのつど
    /// disk I/O をしない、既存 `saved_revision` 比較と同じ形、`Document::revision` の
    /// doc 参照 — transient overlay はここに含まれないので、ドラッグ中の途中経過だけを
    /// 理由に自動保存が走ることはない)。
    ///
    /// # 未保存の新規 project(`project_path` が無い)
    ///
    /// **何もせず `Ok(None)`**(AE 先例: Auto-Save は保存済みの project の隣にしか
    /// 書けない — 一度も明示 Save していない project を自動でどこかへ書き始めると、
    /// 利用者が知らない場所にファイルが生える)。呼び手は最初の明示 Save が起きるまで
    /// 自動保存を素通りさせればよい([`crate::Document`] は自分の path を持たない設計
    /// なので、この判定は呼び手から渡された `Option` の形で表す)。
    ///
    /// # atomic 書き込み
    ///
    /// 実際の書き込みは [`Self::save_atomic`](同一ディレクトリへ `.tmp` → `rename`)。
    /// 書き込み中に落ちても、上書き対象だった直前の世代ファイルは無傷のまま残る。
    pub fn auto_save(
        &self,
        project_path: Option<&Path>,
        since: &Revision,
        config: &AutoSaveConfig,
    ) -> Result<Option<PathBuf>, StoreError> {
        let Some(project_path) = project_path else {
            return Ok(None);
        };
        if self.revision() == *since {
            return Ok(None);
        }

        let dir = Self::auto_save_dir(project_path);
        std::fs::create_dir_all(&dir).map_err(|e| StoreError::Io(e.to_string()))?;

        let stem = Self::auto_save_stem(project_path);
        let ext = Self::auto_save_ext(project_path);
        let generations = config.generations.max(1);

        let mut existing = Self::existing_generations(&dir, &stem, &ext)?;
        let next_seq = existing.last().map(|(seq, _)| seq + 1).unwrap_or(1);
        let target = Self::generation_path(&dir, &stem, &ext, next_seq);

        self.save_atomic(&target)?;

        existing.push((next_seq, target.clone()));
        if existing.len() > generations {
            let overflow = existing.len() - generations;
            for (_, stale) in existing.iter().take(overflow) {
                // ベストエフォート。消せなくても直近の保存自体は成功しているので
                // ここで失敗させない(次回また削除を試みる)。
                let _ = std::fs::remove_file(stale);
            }
        }

        Ok(Some(target))
    }

    /// `<project 隣>/<name> auto-save/`。発注書の指定どおり(AE 慣習)。
    pub fn auto_save_dir(project_path: &Path) -> PathBuf {
        let stem = Self::auto_save_stem(project_path);
        let parent = project_path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        parent.join(format!("{stem} auto-save"))
    }

    fn auto_save_stem(project_path: &Path) -> String {
        project_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled")
            .to_owned()
    }

    /// 先頭 `.` 込みの拡張子(`.motolii`)。無ければ空文字列。
    fn auto_save_ext(project_path: &Path) -> String {
        project_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{e}"))
            .unwrap_or_default()
    }

    fn generation_path(dir: &Path, stem: &str, ext: &str, seq: u64) -> PathBuf {
        dir.join(format!("{stem}.autosave-{seq}{ext}"))
    }

    fn generation_prefix(stem: &str) -> String {
        format!("{stem}.autosave-")
    }

    /// ファイル名から `seq` を読む。tmp ファイル(先頭 `.`)や無関係なファイルは
    /// `None` を返し、ローテーションの勘定に混ぜない — 自動保存が中断して tmp が
    /// 残骸として残っていても、これが世代として数えられて正本を押し出すことはない。
    fn parse_generation_seq(file_name: &str, stem: &str, ext: &str) -> Option<u64> {
        let prefix = Self::generation_prefix(stem);
        let rest = file_name.strip_prefix(prefix.as_str())?;
        let rest = if ext.is_empty() {
            rest
        } else {
            rest.strip_suffix(ext)?
        };
        rest.parse::<u64>().ok()
    }

    /// 既存の世代ファイルを `seq` 昇順(古い順)で返す。ディレクトリの中身だけから
    /// 導く — 別に state を持たない。
    fn existing_generations(
        dir: &Path,
        stem: &str,
        ext: &str,
    ) -> Result<Vec<(u64, PathBuf)>, StoreError> {
        let mut out = Vec::new();
        if !dir.exists() {
            return Ok(out);
        }
        for entry in std::fs::read_dir(dir).map_err(|e| StoreError::Io(e.to_string()))? {
            let entry = entry.map_err(|e| StoreError::Io(e.to_string()))?;
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if let Some(seq) = Self::parse_generation_seq(&name, stem, ext) {
                out.push((seq, entry.path()));
            }
        }
        out.sort_by_key(|(seq, _)| *seq);
        Ok(out)
    }
}

/// 自動保存の既定値。**Settings が読める形の置き場**(まだ Settings 側は未接続 —
/// shell 結線が済んだら `next/ui/motolii-settings-pane` の自動保存間隔 UI から
/// ここの値を上書きして `Document::auto_save` へ渡す想定)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AutoSaveConfig {
    /// 自動保存の間隔(秒)。**タイマー自体は shell の subscription 側が引く** —
    /// ここは値の置き場でしかない(`Document::auto_save` は間隔を計らない)。
    pub interval_secs: u64,
    /// 保持世代数。これを超えた古い世代は自動保存のたびに削除される。
    pub generations: usize,
}

impl AutoSaveConfig {
    /// 既定間隔: 5分(AE `Auto-Save` の既定 `Save Every` に合わせた、発注書の指定)。
    pub const DEFAULT_INTERVAL_SECS: u64 = 5 * 60;
    /// 既定保持世代数: 5(AE `Auto-Save` の既定 `Maximum Project Versions`)。
    pub const DEFAULT_GENERATIONS: usize = 5;
}

impl Default for AutoSaveConfig {
    fn default() -> Self {
        Self {
            interval_secs: Self::DEFAULT_INTERVAL_SECS,
            generations: Self::DEFAULT_GENERATIONS,
        }
    }
}
