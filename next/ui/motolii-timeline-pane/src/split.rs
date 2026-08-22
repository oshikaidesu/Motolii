//! Split(レイヤー分割) — 高 freq の未実装動詞(B39)。発注: 裁定189 dispatch
//! 「Split(レイヤー分割)— 高 freq の未実装動詞(B39)」。
//!
//! ## map 消化(`next/reference/normal-map.tsv` bundle 列 B39)
//!
//! **消化**(この module の `split_plan`/`split_selected_plan` が意味を持つ、
//! いずれも同一の Split 動詞への別名/別入口):
//! - id 163 `Split / Razor (clip at playhead)` — 本体(playhead で1層を割る)
//! - id 171 `Add Edit`(Razor 相当、単層) / id 803 同名重複(misc) / id 1046
//!   同名重複(playback) — 163 と同じ意味(単層 split の別名、二次資料側の
//!   同義語)
//! - id 267 `Razor(split)`(Command+B) — 163 のショートカット別名。実キー
//!   割当は shell 側(次波、下記「統合手順」)
//! - id 317 `Split selected layers` — [`split_selected_plan`](選択レイヤー
//!   全部への一括 split・1 apply_all = 1 undo、EXACT TARGET 3)
//!
//! **見送り**(理由つき — この2関数では意味が組めない/範囲外):
//! - id 172 `Add Edit to All Tracks` / id 804 同名重複(misc) / id 1047
//!   同名重複(playback) — **選択レイヤーではなく「全トラック」を割る**動詞。
//!   EXACT TARGET 3 は「選択中レイヤー全部」であって「comp の全 layer」では
//!   ないので、意味が違う(選択が空でも発火する動詞は別発注 — `layers`
//!   引数を空にすれば同じ関数を使い回せるが、「全レイヤー」を渡す判断は
//!   呼び出し側/次波の仕事)。
//! - id 182 `Blade Edit Mode` / id 183 `Blade tool` / id 247 `Normal Edit Mode`
//!   / id 265 `Range Selection Mode` / id 266 `Razor Tool` — **ツールモード**
//!   (ポインタの意味を切り替える持続状態)であって、1回の split 操作ではない。
//!   `split_plan`/`split_selected_plan` は純関数(1呼び出し=1判断)なので
//!   モードという概念自体が乗らない — モード導入は別発注(shell 側の
//!   カーソル/クリック解釈の話であって、この store 読み専用 module の範囲外)。
//! - id 198 `Dynamic Trim mode` / id 199 `Edit Point Type` — trim 系(この
//!   pane は `clip_gesture.rs` の Edge drag で trim を既に実装済み・採用済み。
//!   Split とは別動詞)。
//! - id 67 `Trim Audio Edits to Frame Boundaries` — 音声編集点のフレーム境界
//!   スナップ精度機能。split ではなく trim 精度の話で、この発注の範囲外。
//! - id 326-337(Trim Edit/Trim Start/Trim End/Trim Mode/Trim tool 等) —
//!   既に 採用済み(`clip_gesture.rs` の `BarPart::EdgeIn`/`EdgeOut`)。この
//!   発注が触るのは Split だけで、Trim 側は無改修。
//!
//! ## `split_plan` の契約
//!
//! [`split_plan`] は **純関数** — `doc_view`(読み取り専用の [`StoreView`])を
//! 読むだけで、Document を書かない。返すのは「これを1つの `apply_all` に
//! 通せば split が成立する」という **既存 [`Intent`] の列**(新しい Intent
//! 種別は増やしていない — 発注の「新 Intent 禁止」)。
//!
//! - playhead(`at_frame`)が layer の実クリップ区間 `[in, out)`
//!   (`LayerTiming::covers`)の **内側**(端を含まない)の時だけ `Ok`。
//!   端(`at_frame == in` または `at_frame == out`)は「切っても片方が
//!   duration=0 になるだけで何も割れていない」ので `Err`(no-op の理由)。
//! - `layer` が `locked`([`motolii_store::LayerAttrs::locked`])なら `Err`
//!   (他の層変更 Intent と同じ柵を、書く前にここで先取りして理由を返す)。
//! - **frozen はここでは先取りしない** — `Document::apply_all` が原子的
//!   (1つでも `Err` なら batch 全体を無かったことにする、`document.rs`
//!   `apply_all` の doc 参照)なので、凍結中の部分木への split は
//!   `doc.apply_all(...)` 呼び出し自体が理由つき `Err` を返す形で自然に
//!   拒まれる。二重に検査ロジックを持たせない(store の柵を pane 側で
//!   複製しない、既存の「store に聞く」流儀)。
//!
//! `Ok` の場合、返す列は:
//! 1. 元 layer の `Intent::SetTiming` — `start`/`source_in`/`speed` は不変、
//!    `duration` だけ `at_frame - start` へ縮める(前半だけ残す)。
//! 2. 複製 layer(`doc_view.next_layer_id()` で新規採番)の
//!    `Intent::AddLayer` + `Intent::SetMeta`(`source`/`order` は元のまま、
//!    `timing` は `start=at_frame`・`duration=out-at_frame`・`source_in` は
//!    **元 timing の `source_frame(at_frame)`**(裁定63 の speed 込み写像 —
//!    `source_in + (at_frame - start)` の 1:1 決め打ちを再現しない)・
//!    `speed` は元のまま)。
//! 3. 複製 layer の `Intent::SetAttrs`(`frozen` 以外の全フィールドを元
//!    `LayerAttrs` から複製。`frozen` は [`Intent::Freeze`]/[`Intent::Unfreeze`]
//!    専用の口で `LayerAttrsPatch` に無いフィールドなので、複製後は既定
//!    `false` — 分割で複製した子が親の凍結を勝手に引き継ぐ方が驚き)。
//!    `locked` は複製先には必ず `false` を書く(元は非 locked であることを
//!    上で確認済み、複製も非 locked が自然)。
//! 4. 元 layer が持つ全 property(`doc_view.properties`)を複製 layer へ
//!    複製 — `PropertySource::Track` なら `Intent::SetTrack`、`Slot` なら
//!    `Intent::SetPropertySlot`。**キーフレームは時刻不変で両半分に残る**
//!    (AE の Split Layer と同じ挙動 — トラックを cut/trim しない。評価時に
//!    `LayerTiming::covers` が範囲外を弾くので、見た目には「その半分の
//!    区間だけ」が効く)。mask の形状/不透明度トラックや effect の param
//!    トラックも `mask.{id}.*`/`effect.{id}.*` という平坦な property名
//!    (`property` module doc 参照)なので、この一括複製に自然に乗る。
//! 5. `Intent::SetMasks`/`Intent::SetEffects`/`Intent::SetShapes`/
//!    `Intent::SetTextDocument` — マスク一覧・effect 一覧・shape 列・
//!    text document は property 束とは別 component(`view.rs` の doc
//!    参照)なので、それぞれの丸ごと複製 Intent を足す(空なら発行しない
//!    — 空の component を新規で書く意味が無い)。
//!
//! ## `split_selected_plan` の契約(EXACT TARGET 3: 選択レイヤー一括)
//!
//! [`split_selected_plan`] は `layers` の各要素へ [`split_plan`] を呼び、
//! 成立した分だけ1本の `Vec<Intent>` へ連結して返す。呼び出し側は**この
//! 1本を1回の `doc.apply_all(...)` に通すだけで「1操作 = 1 undo」が成立する**
//! (`Document::apply_all` の doc「1操作 = 1 undo」節どおり)。
//!
//! **設計判断(この発注書に明示は無いが、正しく動くために必要な決定)**:
//!
//! - **一部の layer が split できなくても batch 全体を諦めない** —
//!   `split_plan` が `Err` を返した layer は黙って skip し、他の layer は
//!   split する。AE の "Split Layer"(選択が複数)と同じ挙動 — playhead が
//!   通っていない layer・locked な layer が選択に混ざっていても、割れる
//!   layer だけ割れる。**全部が `Err` なら**(1つも割れる layer が無い)
//!   全体を `Err` で返す(「何も起きなかった」を無反応ゼロの理由付きで
//!   伝える、M13)。
//! - **新規 layer id の衝突を避ける採番のやり直し** — `split_plan` は毎回
//!   独立に `doc_view.next_layer_id()`(まだ何も適用していない同一
//!   snapshot を読むので、複数回呼んでも常に同じ値)で複製先 id を決める。
//!   複数 layer をまとめて割ると、素朴に `split_plan` を呼び重ねただけでは
//!   **全ての複製が同じ id を要求して衝突する**(2枚目以降の `AddLayer` が
//!   同じ id を再利用しようとする)。ここでは `split_plan` の結果に含まれる
//!   `AddLayer` の id を見つけ、呼び出しごとに採番し直した新しい id へ
//!   [`remap_new_layer_id`] で付け替えてから連結する — `split_plan` の
//!   3引数シグネチャ(発注書どおり)は変えず、複数呼び出しの間だけこの
//!   module が帳尻を合わせる。
//!
//! ## `write::Message` への統合(TL4 統合手順、裁定189 追いつきターン)
//!
//! `Message::SplitAtPlayhead` は **この module ではなく `crate::write::Message`**
//! に住む(pane crate の唯一の公開 `Message` に統合済み — この module が
//! 独自の pane-local `Message` を宣言していた旧版は削除した。重複させない)。
//! `PaneState::update` の match arm(`PaneState::split_at_playhead`)が
//! `session.selected_layers` が空でなければそれを、空なら
//! `session.selection` の単一選択を `layers` として
//! [`split_selected_plan`] を呼び、`Ok(intents)` なら `doc.apply_all(intents)`
//! (失敗時は理由文字列をそのまま返す、既存腕と同じ形 —
//! `delete_selected_keys`/`finish_drag` と同型)。選択が空なら「選択が無いので
//! 分割できない」を理由つきで返す(M13)。
//!
//! **shell 側の結線は未着手**(次波・このレーンの RETURN 参照): キー割当
//! (map id 267 `Command+B`)とメニュー項目(id 163/317 相当)を
//! `Message::Timeline(motolii_timeline_pane::Message::SplitAtPlayhead)` へ
//! 結線するのは `motolii-shell` 側の仕事(`ToggleFold`/`ToggleLoop` と同じ
//! 「shell 先取りなしで `PaneState::update` が完結する」形 — 5例外には
//! 数えない)。id 172/804/1047(Add Edit to All Tracks)を将来採るなら、
//! `split_selected_plan(&doc.view(), &doc.view().layers(), at_frame)`
//! (選択の代わりに全 layer を渡す)で同じ関数を使い回せる — 新しい関数は
//! 要らない(map 消化ノートの見送り理由と対で読むこと)。

use motolii_store::{
    Intent, LayerAttrsPatch, LayerId, LayerMeta, LayerTiming, PropertyBase, StoreView,
};

/// 1 layer を `at_frame` で split する Intent 列を組む(モジュール doc 「`split_plan`
/// の契約」参照)。**純関数** — `doc_view` を読むだけで Document は書かない。
pub fn split_plan(
    doc_view: &StoreView<'_>,
    layer: LayerId,
    at_frame: i64,
) -> Result<Vec<Intent>, String> {
    let meta = doc_view
        .meta(layer)
        .map_err(|error| format!("layer {} の meta を読めない: {error}", layer.0))?
        .ok_or_else(|| format!("layer {} に素材が置かれていないので分割できない", layer.0))?;

    let timing = meta.timing;
    let in_frame = timing.start;
    let out_frame = timing.start + timing.duration;

    if at_frame <= in_frame || at_frame >= out_frame {
        return Err(format!(
            "playhead(frame {at_frame})が layer {} のクリップ区間 [{in_frame}, {out_frame}) の\
             内側にない — 端(in/out)では分割できない",
            layer.0
        ));
    }

    let attrs = doc_view
        .attrs(layer)
        .map_err(|error| format!("layer {} の attrs を読めない: {error}", layer.0))?
        .unwrap_or_default();
    if attrs.locked {
        return Err(format!(
            "layer {} はロックされているので分割できない(先に locked を外すこと)",
            layer.0
        ));
    }

    let mut intents = Vec::new();

    // 1) 元 layer は前半だけ残す(start/source_in/speed 不変、duration だけ縮む)。
    intents.push(Intent::SetTiming {
        layer,
        timing: LayerTiming {
            start: timing.start,
            duration: at_frame - timing.start,
            source_in: timing.source_in,
            speed: timing.speed,
        },
    });

    // 2) 複製 layer を at 以降に置く。source_in は speed 込みの写像を使う
    //    (`LayerTiming::source_frame` — 裁定63 の穴を再現しない)。
    let new_layer = LayerId(doc_view.next_layer_id());
    let source_in_at_cut = timing.source_frame(at_frame).ok_or_else(|| {
        format!(
            "layer {} の frame {at_frame} が区間 [{in_frame}, {out_frame}) の外(内部矛盾)",
            layer.0
        )
    })?;
    intents.push(Intent::AddLayer(new_layer));
    intents.push(Intent::SetMeta {
        layer: new_layer,
        meta: LayerMeta {
            source: meta.source.clone(),
            order: meta.order,
            timing: LayerTiming {
                start: at_frame,
                duration: out_frame - at_frame,
                source_in: source_in_at_cut,
                speed: timing.speed,
            },
        },
    });

    // 3) attrs を複製(frozen を除く — モジュール doc 参照)。locked は必ず false。
    intents.push(Intent::SetAttrs {
        layer: new_layer,
        patch: LayerAttrsPatch {
            hidden: Some(attrs.hidden),
            parent: Some(attrs.parent),
            blend_mode: Some(attrs.blend_mode),
            matte: Some(attrs.matte),
            name: Some(attrs.name.clone()),
            auto_orient: Some(attrs.auto_orient),
            pinned: Some(attrs.pinned),
            solo: Some(attrs.solo),
            locked: Some(false),
            label_color: Some(attrs.label_color),
        },
    });

    // 4) track 群複製 — キーフレームは時刻不変で両半分に残る(AE 型)。
    for property in doc_view.properties(layer) {
        let source = doc_view.property_source(layer, &property).map_err(|error| {
            format!(
                "layer {} の property `{}` を読めない: {error}",
                layer.0,
                property.name()
            )
        })?;
        // base(Track/Slot、無ければ何も書かない)と modulators(裁定213 の加算列、
        // 空でなければ複製)は独立な軸——両方揃った property も、旧 Link 相当
        // (base 無し・modulator 1本)の property も、同じ形で複製できる。
        // modulator は**参照先を書き換えない**で複製する — 分割の後半分も
        // 元と同じ source を追い続けるのが「切っても意味は変わらない」
        // という分割の約束に合う(参照先を新 layer へ張り替えると、
        // 後半分だけ別の物に追従し始めて意味が割れる)。
        if let Some(source) = source {
            match source.base {
                Some(PropertyBase::Track(track)) => {
                    intents.push(Intent::SetTrack {
                        layer: new_layer,
                        property: property.clone(),
                        track,
                    });
                }
                Some(PropertyBase::Slot(slot)) => {
                    intents.push(Intent::SetPropertySlot {
                        layer: new_layer,
                        property: property.clone(),
                        slot,
                    });
                }
                None => {}
            }
            if !source.modulators.is_empty() {
                intents.push(Intent::SetPropertyModulators {
                    layer: new_layer,
                    property,
                    modulators: source.modulators,
                });
            }
        }
    }

    // 5) masks/effects/shapes/text は別 component — 空でなければ丸ごと複製する。
    let masks = doc_view
        .masks(layer)
        .map_err(|error| format!("layer {} の masks を読めない: {error}", layer.0))?;
    if !masks.is_empty() {
        intents.push(Intent::SetMasks { layer: new_layer, masks });
    }
    let effects = doc_view
        .effects(layer)
        .map_err(|error| format!("layer {} の effects を読めない: {error}", layer.0))?;
    if !effects.is_empty() {
        intents.push(Intent::SetEffects { layer: new_layer, effects });
    }
    let shapes = doc_view
        .shapes(layer)
        .map_err(|error| format!("layer {} の shapes を読めない: {error}", layer.0))?;
    if !shapes.is_empty() {
        intents.push(Intent::SetShapes { layer: new_layer, shapes });
    }
    if let Some(document) = doc_view
        .text_document(layer)
        .map_err(|error| format!("layer {} の text を読めない: {error}", layer.0))?
    {
        intents.push(Intent::SetTextDocument { layer: new_layer, document });
    }

    Ok(intents)
}

/// 選択レイヤー全部への一括 split(EXACT TARGET 3)。呼び出し側はこの1本を
/// **1回の `doc.apply_all(...)` に通すだけで良い**(1 apply_all = 1 undo)。
/// 割れなかった layer(playhead が区間外・locked)は黙って skip する
/// (モジュール doc「設計判断」参照) — 1つも割れなければ `Err`。
pub fn split_selected_plan(
    doc_view: &StoreView<'_>,
    layers: &[LayerId],
    at_frame: i64,
) -> Result<Vec<Intent>, String> {
    let mut next_new_id = doc_view.next_layer_id();
    let mut intents = Vec::new();

    for &layer in layers {
        let Ok(layer_intents) = split_plan(doc_view, layer, at_frame) else {
            continue;
        };
        let assigned_id = layer_intents.iter().find_map(|intent| match intent {
            Intent::AddLayer(id) => Some(*id),
            _ => None,
        });
        let Some(assigned_id) = assigned_id else {
            // split_plan は必ず AddLayer を含む Ok を返す契約 — 契約違反時は
            // この layer だけ安全側で捨てる(他の layer の split は続ける)。
            continue;
        };
        let new_id = LayerId(next_new_id);
        next_new_id += 1;
        intents.extend(remap_new_layer_id(layer_intents, assigned_id, new_id));
    }

    if intents.is_empty() {
        return Err(format!(
            "選択レイヤーのうち、frame {at_frame} で分割できるものが無い \
             (playhead が区間外、または全部 locked)"
        ));
    }
    Ok(intents)
}

/// `split_selected_plan` が複数 layer をまとめる時、それぞれの `split_plan` が
/// 独立に採番した複製先 id(`old_id`)を、衝突しない新しい id(`new_id`)へ
/// 付け替える(モジュール doc「設計判断」参照)。`split_plan` が返す Intent の
/// 閉じた集合だけを扱う(網羅 match ではなく `other` 受け皿 — `split_plan` が
/// 将来別の Intent 種別を足しても、ここは黙って通すだけで壊れない)。
fn remap_new_layer_id(intents: Vec<Intent>, old_id: LayerId, new_id: LayerId) -> Vec<Intent> {
    if old_id == new_id {
        return intents;
    }
    intents
        .into_iter()
        .map(|intent| match intent {
            Intent::AddLayer(id) if id == old_id => Intent::AddLayer(new_id),
            Intent::SetMeta { layer, meta } if layer == old_id => {
                Intent::SetMeta { layer: new_id, meta }
            }
            Intent::SetAttrs { layer, patch } if layer == old_id => {
                Intent::SetAttrs { layer: new_id, patch }
            }
            Intent::SetTrack { layer, property, track } if layer == old_id => {
                Intent::SetTrack { layer: new_id, property, track }
            }
            Intent::SetPropertySlot { layer, property, slot } if layer == old_id => {
                Intent::SetPropertySlot { layer: new_id, property, slot }
            }
            Intent::SetMasks { layer, masks } if layer == old_id => {
                Intent::SetMasks { layer: new_id, masks }
            }
            Intent::SetEffects { layer, effects } if layer == old_id => {
                Intent::SetEffects { layer: new_id, effects }
            }
            Intent::SetShapes { layer, shapes } if layer == old_id => {
                Intent::SetShapes { layer: new_id, shapes }
            }
            Intent::SetTextDocument { layer, document } if layer == old_id => {
                Intent::SetTextDocument { layer: new_id, document }
            }
            other => other,
        })
        .collect()
}
