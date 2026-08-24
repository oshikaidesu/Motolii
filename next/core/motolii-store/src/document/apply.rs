//! `Intent` の全枝(26)の適用本体(`Document::write`)。`document.rs` から
//! 移送(裁定220 SP-3、中身は変えていない)。

use re_types_core::SerializedComponentBatch;

use crate::components::{
    descriptor_assets, descriptor_attrs, descriptor_composition, descriptor_effects,
    descriptor_markers, descriptor_masks, descriptor_meta, descriptor_present, descriptor_shapes,
    descriptor_slots, descriptor_text, descriptor_track, LayerPresent, TrackJson,
};
use crate::slot::PropertySource;
use crate::StoreError;

use super::validate::{
    check_not_frozen, check_not_locked, freeze_attrs_batch, is_frozen_or_within_frozen,
    validate_masks_have_shapes, validate_no_link_cycle, validate_no_parent_cycle,
};
use super::{Document, Intent};

impl Document {
    /// 唯一の意味づけ書き口。`pub(crate)` なのは `persist.rs` が `AddLayer` を
    /// 同じ edit 刻みで書くため(履歴を畳む = 1 tick にまとめる、裁定56)。
    pub(crate) fn write(&mut self, intent: Intent, at: i64) -> Result<(), StoreError> {
        let batches = match intent {
            Intent::AddLayer(layer) => (layer.entity_path(), vec![serialize_present(true)?]),
            Intent::RemoveLayer(layer) => {
                // 削除は最も破壊的な編集(元に戻すには undo するしかない)。locked は
                // 他の層変更 Intent と同じく理由つき Err で拒む(supervisor 裁定、
                // AE と同じ意味論)。解除→削除の2手は常に可能 — `check_not_locked` は
                // `locked` 自身の解除/再ロックだけを別扱いする `SetAttrs` を経由しない。
                check_not_locked(&self.view(), layer)?;
                check_not_frozen(&self.view(), layer)?;
                (layer.entity_path(), vec![serialize_present(false)?])
            }
            Intent::SetComposition(composition) => {
                let json = serde_json::to_string(&composition)?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_composition(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetMarkers { markers } => {
                let json = serde_json::to_string(&markers)?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_markers(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetMasks { layer, masks } => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen(&self.view(), layer)?;
                crate::mask::validate_unique_ids(&masks)?;
                validate_masks_have_shapes(&self.view(), layer, &masks)?;
                let json = serde_json::to_string(&masks)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_masks(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::AddMask { layer, mask, shape } => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen(&self.view(), layer)?;
                let mut masks = self.view().masks(layer)?;
                masks.push(mask);
                crate::mask::validate_unique_ids(&masks)?;
                let masks_json = serde_json::to_string(&masks)?;
                let shape_property = crate::PropertyId::mask_shape(mask.id);
                let shape_json = serde_json::to_string(&PropertySource::track(shape))?;
                // **1つの chunk として同時に ingest する**(下の `(path, batches)` を
                // 呼び出し元の `write()` 末尾がまとめて1回で書く) — 「一覧だけ更新
                // されて shape が無い」瞬間が物理的に存在しない(2つの intent の
                // 順序に頼る `apply_all([SetMasks, SetTrack])` との違い、上記
                // `Intent::SetMasks`/`Intent::AddMask` の doc 参照)。
                (
                    layer.entity_path(),
                    vec![
                        SerializedComponentBatch {
                            descriptor: descriptor_masks(),
                            array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(
                                masks_json,
                            )])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                        },
                        SerializedComponentBatch {
                            descriptor: descriptor_track(&shape_property),
                            array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(
                                shape_json,
                            )])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                        },
                    ],
                )
            }
            Intent::SetTiming { layer, timing } => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen(&self.view(), layer)?;
                // meta の一部なので、読んで差し替えて書き戻す。
                // **専用の component を足さない** — 増やすと読み口も増える。
                let current = self.view().meta(layer)?;
                let Some(mut meta) = current else {
                    return Err(StoreError::Property(format!(
                        "layer {} に素材が置かれていないので配置を決められない",
                        layer.0
                    )));
                };
                meta.timing = timing;
                let json = serde_json::to_string(&meta)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_meta(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetMeta { layer, meta } => {
                // **新規配置専用**(裁定108(c))。既に meta があるのに丸ごと差し替えを
                // 許すと、呼び手が読まずに組んだ値で timing/source/order のどれかが
                // 黙って戻る事故が構造的に作れてしまう。既存 layer は
                // SetSource/SetOrder/SetTiming のフィールド単位の口を使うこと。
                if self.view().meta(layer)?.is_some() {
                    return Err(StoreError::Property(format!(
                        "layer {} は既に meta を持つ。SetMeta は新規配置専用 — 既存 layer の \
                         素材/重ね順/配置を変えるには SetSource/SetOrder/SetTiming を使うこと",
                        layer.0
                    )));
                }
                let json = serde_json::to_string(&meta)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_meta(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetSource { layer, source } => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen(&self.view(), layer)?;
                let current = self.view().meta(layer)?;
                let Some(mut meta) = current else {
                    return Err(StoreError::Property(format!(
                        "layer {} に meta が無い(先に SetMeta で配置すること)",
                        layer.0
                    )));
                };
                meta.source = source;
                let json = serde_json::to_string(&meta)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_meta(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetOrder { layer, order } => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen(&self.view(), layer)?;
                let current = self.view().meta(layer)?;
                let Some(mut meta) = current else {
                    return Err(StoreError::Property(format!(
                        "layer {} に meta が無い(先に SetMeta で配置すること)",
                        layer.0
                    )));
                };
                meta.order = order;
                let json = serde_json::to_string(&meta)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_meta(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetAttrs { layer, patch } => {
                // 凍結中の部分木への編集は理由つき拒否(裁定119)。`layer` 自身の
                // frozen 状態は見ない(`LayerAttrs::frozen`/`check_not_frozen` の doc
                // 参照) — 凍結された Group 自身の attrs(名前を変える・移動する等)は
                // ここでは拒まない。
                check_not_frozen(&self.view(), layer)?;
                // `parent` を凍結中の Group(またはその部分木の中)へ向けようとしていな
                // いかも確かめる。`check_not_frozen` は `layer` 自身の祖先だけを見るので、
                // 「今は凍結の外に居る layer を、凍結中の Group の新しい子として迎え
                // 入れる」経路はここが無いと素通りしてしまう(新しい子を迎えるのも
                // 部分木の中身を変える編集の一種)。
                if let Some(Some(new_parent)) = patch.parent {
                    if is_frozen_or_within_frozen(&self.view(), new_parent)? {
                        return Err(StoreError::Property(format!(
                            "layer {} の parent を layer {} にはできない — \
                             凍結中(frozen)のグループか、その部分木の中にある \
                             (先に unfreeze すること)",
                            layer.0, new_parent.0
                        )));
                    }
                }
                // read-modify-write — `attrs` が無い layer への初回書き込みは
                // `LayerAttrs::default()` を土台にする(`meta` と違い、属性は元々
                // 省略可能なので「まだ無い」ことがエラーではない)。
                let current = self.view().attrs(layer)?.unwrap_or_default();
                // locked は `locked` 自身の解除(または再ロック)だけ常に通す —
                // 他のどれか1つでも `Some` なら「locked 以外のフィールド」を触ろうと
                // しているので拒む。自分をロックしたら二度と触れなくなる詰みを
                // 作らないよう、`locked` を触るだけの patch は現在の locked 状態に
                // 関わらず素通しする。
                if current.locked {
                    let touches_other_than_locked = patch.hidden.is_some()
                        || patch.parent.is_some()
                        || patch.blend_mode.is_some()
                        || patch.matte.is_some()
                        || patch.name.is_some()
                        || patch.auto_orient.is_some()
                        || patch.pinned.is_some()
                        || patch.solo.is_some()
                        || patch.label_color.is_some();
                    if touches_other_than_locked {
                        return Err(StoreError::Property(format!(
                            "layer {} は locked なので attrs を変更できない(先に \
                             locked を外すこと)",
                            layer.0
                        )));
                    }
                }
                if let Some(new_parent) = patch.parent {
                    validate_no_parent_cycle(&self.view(), layer, new_parent)?;
                }
                let attrs = patch.apply_to(current);
                let json = serde_json::to_string(&attrs)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_attrs(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetEffects { layer, effects } => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen(&self.view(), layer)?;
                crate::effect::validate_unique_ids(&effects)?;
                let json = serde_json::to_string(&effects)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_effects(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetShapes { layer, shapes } => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen(&self.view(), layer)?;
                let json = serde_json::to_string(&shapes)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_shapes(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetTextDocument { layer, document } => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen(&self.view(), layer)?;
                crate::text::validate(&document)?;
                let json = serde_json::to_string(&document)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_text(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetTrack {
                layer,
                property,
                track,
            } => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen(&self.view(), layer)?;
                // **`PropertySource::track()` でラップして書く**(`slot` 発注単位)。
                // 裁定213 で wire 形は明示的な `{"base":...,"modulators":[]}` に
                // なった(bit単位で裸 `KeyframeTrack` と同じではない)が、読み口
                // (`view.track()`)は旧形式ごと後方互換で読むので既存の呼び手は
                // 変わらない。**丸ごと上書き**(modulator も含めて消える) —
                // `Slot`/`Link` から普通の track へ戻す時に「専用の解除操作は
                // 要らない」という既存の設計(このファイル各所の doc 参照)を
                // modulator にもそのまま適用する。modulator だけを差し替えたい
                // 呼び手は [`Intent::SetPropertyModulators`] を使うこと。
                let json = serde_json::to_string(&PropertySource::track(track))?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_track(&property),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetCameraTrack { property, track } => {
                let json = serde_json::to_string(&PropertySource::track(track))?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_track(&property),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetPropertySlot {
                layer,
                property,
                slot,
            } => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen(&self.view(), layer)?;
                let json = serde_json::to_string(&PropertySource::slot(slot))?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_track(&property),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetPropertyLink {
                layer,
                property,
                link,
            } => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen(&self.view(), layer)?;
                validate_no_link_cycle(&self.view(), layer, &property, &link)?;
                // **裁定213**: 加算が「置き換え」を包含する — base を持たず
                // modulator 1本だけの形(`link_only`)は、旧 `PropertySource::Link`
                // と全く同じ値を返す(`None` + x = x)。この Intent の見た目の
                // 挙動(この property を丸ごと link の値にする)は変えない。
                let json = serde_json::to_string(&PropertySource::link_only(link))?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_track(&property),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetCameraPropertySlot { property, slot } => {
                let json = serde_json::to_string(&PropertySource::slot(slot))?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_track(&property),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetPropertyModulators {
                layer,
                property,
                modulators,
            } => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen(&self.view(), layer)?;
                for link in &modulators {
                    validate_no_link_cycle(&self.view(), layer, &property, link)?;
                }
                // **`base` は読んで保つ**(`SetAttrs` と同じ「現在を読んでから
                // 該当フィールドだけ差し替える」形) — `SetPropertyLink`/
                // `SetTrack`/`SetPropertySlot` の「丸ごと置き換え」とは違う口。
                let mut source = self
                    .view()
                    .property_source(layer, &property)?
                    .unwrap_or(PropertySource {
                        base: None,
                        modulators: Vec::new(),
                    });
                source.modulators = modulators;
                let json = serde_json::to_string(&source)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_track(&property),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetCameraPropertyModulators {
                property,
                modulators,
            } => {
                // カメラの property には layer が無いので、循環検査の起点は
                // `Document::composition_path()` を指す仮想の layer id を持たない
                // ——`validate_no_link_cycle` は `(LayerId, PropertyId)` を鍵にする
                // ため、カメラ自身が modulator の参照先(source_layer/source_property)
                // に選ばれることは無い(`PropertyLink::source_layer` は常に
                // 実在 layer を指す設計、`SetCameraPropertyModulators` 自身は
                // 「カメラの property を起点とする」循環しか気にする必要が無い)。
                // カメラを指す循環は「カメラ自身が link 元になる」経路が無い
                // ので構造的に発生しない——検査は省略してよい
                // (`SetCameraTrack`/`SetCameraPropertySlot` も同様に検査を持たない)。
                let mut source = self
                    .view()
                    .camera_property_source(&property)?
                    .unwrap_or(PropertySource {
                        base: None,
                        modulators: Vec::new(),
                    });
                source.modulators = modulators;
                let json = serde_json::to_string(&source)?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_track(&property),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetSlots { slots } => {
                crate::slot::validate_unique_ids(&slots)?;
                let json = serde_json::to_string(&slots)?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_slots(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::AdmitAsset { draft } => {
                // read-modify-write — `SetTiming`/`SetSource` と同じ形(裁定162):
                // 現在の台帳を読み、`admit` の重複統合を経てから丸ごと書き戻す。
                let mut table = self.view().assets_table()?;
                table
                    .admit(draft)
                    .map_err(|e| StoreError::Property(e.to_string()))?;
                let json = serde_json::to_string(&table)?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_assets(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::RemoveAsset { asset } => {
                let mut table = self.view().assets_table()?;
                table
                    .remove(asset)
                    .map_err(|e| StoreError::Property(e.to_string()))?;
                let json = serde_json::to_string(&table)?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_assets(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::RelinkAsset {
                asset,
                path_absolute,
                project_root,
            } => {
                let mut table = self.view().assets_table()?;
                let path = std::path::Path::new(&path_absolute);
                table
                    .relink(
                        asset,
                        path,
                        project_root.as_deref().map(std::path::Path::new),
                    )
                    .map_err(|e| StoreError::Property(e.to_string()))?;
                let json = serde_json::to_string(&table)?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_assets(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::Freeze { group } => {
                check_not_locked(&self.view(), group)?;
                check_not_frozen(&self.view(), group)?;
                freeze_attrs_batch(&self.view(), group, true)?
            }
            Intent::Unfreeze { group } => {
                check_not_locked(&self.view(), group)?;
                check_not_frozen(&self.view(), group)?;
                freeze_attrs_batch(&self.view(), group, false)?
            }
        };

        let (path, batches) = batches;
        self.ingest(path, batches, at)
    }

}

fn serialize_present(present: bool) -> Result<SerializedComponentBatch, StoreError> {
    Ok(SerializedComponentBatch {
        descriptor: descriptor_present(),
        array: <LayerPresent as re_types_core::Loggable>::to_arrow([LayerPresent(present)])
            .map_err(|e| StoreError::Chunk(e.to_string()))?,
    })
}
