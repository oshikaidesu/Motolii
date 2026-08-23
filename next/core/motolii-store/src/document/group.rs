//! G1(裁定174)グループ化/グループ解除動詞(`Document::group_layers`/
//! `Document::ungroup_layers`)と、ungroup が使う変換の焼き込み。`document.rs`
//! から移送(裁定220 SP-3、中身は変えていない)。

use motolii_core::RationalTime;
use motolii_eval::{Interp, Keyframe, KeyframeTrack, Value};

use crate::view::StoreView;
use crate::{LayerAttrsPatch, LayerMeta, LayerSource, LayerTiming, StoreError};

use super::{Document, Intent, LayerId, PropertyId};

impl Document {
    /// G1(裁定174「意図優先の原則」)グループ化動詞 — ⌘G。
    ///
    /// 選択された `layers` 全部の parent を、新しく生む `LayerSource::Group`
    /// layer へ向ける。**既存語彙(`AddLayer`+`SetMeta`+`SetAttrs`)の合成で足りる**
    /// (裁定174 §2「H3 廃止→G1 へ置換」の設計メモどおり) — 専用 `Intent` は
    /// 増えない。`apply_all` を1回だけ呼ぶので **1 gesture = 1 undo**。
    ///
    /// - 空選択は no-op(`Ok(None)`)。`apply_all` すら呼ばない — 空の undo 刻みを
    ///   積まない
    /// - 単一選択も可(その1層だけを子に持つ Group が生まれる)
    /// - 選択に既存の Group が混じっていても普通に子として括れる(入れ子)
    /// - Group 自身は**単位変換で生まれる**(anchor/position/scale/rotation/skew
    ///   の track を一切書かない — 既定値=恒等)ので、**絵は不変**(裁定174
    ///   OUTCOME)
    /// - `layers` に locked な layer が1つでも含まれていれば、`Intent::SetAttrs`
    ///   の locked 柵(`write` 内、`Intent::SetAttrs` 腕参照)がその1件を `Err`
    ///   にし、`apply_all` の原子性(バッチ全体を無かったことにする)が
    ///   グループ化そのものを取り消す(M13)
    pub fn group_layers(&mut self, layers: &[LayerId]) -> Result<Option<LayerId>, StoreError> {
        if layers.is_empty() {
            return Ok(None);
        }

        let view = self.view();
        let group_id = LayerId(view.next_layer_id());
        let comp_duration = view
            .composition()?
            .map(|composition| composition.duration_frames)
            .unwrap_or(0);

        let mut intents = Vec::with_capacity(layers.len() + 2);
        intents.push(Intent::AddLayer(group_id));
        intents.push(Intent::SetMeta {
            layer: group_id,
            meta: LayerMeta {
                source: LayerSource::Group,
                order: group_id.0 as i16,
                timing: LayerTiming::place(0, None, comp_duration),
            },
        });
        for &child in layers {
            intents.push(Intent::SetAttrs {
                layer: child,
                patch: LayerAttrsPatch {
                    parent: Some(Some(group_id)),
                    ..Default::default()
                },
            });
        }

        self.apply_all(intents)?;
        Ok(Some(group_id))
    }

    /// G1 Ungroup 動詞 — ⌘⇧G。
    ///
    /// `groups` に渡した各 `LayerSource::Group` layer を tombstone にし、その
    /// 直接の子(present な layer で `attrs.parent == Some(group)`)の parent を
    /// **Group 自身の親**(無ければトップレベル = `None`)へ付け替える。
    ///
    /// **子の world 位置は保存される**(裁定174 OUTCOME「Group の変換を子ローカルへ
    /// 焼き込み」) — 数式は [`bake_child_local`] 参照、H1 の正本
    /// ([`StoreView::local_transform`]、`world_affine` と同じ計算の部品)を
    /// 単一源として使い回す。焼き込みが要らない(Group の local が恒等に近い —
    /// 最も普通のケース、`group_layers` が作った直後の Group はまさにこれ)場合は
    /// 子の transform track を一切書き換えない(不要な副作用を避ける)。
    ///
    /// **既知の制限**: baking は `RationalTime::ZERO`(comp の先頭)の1時点で
    /// 評価する。Group 自身がアニメーションしている場合、この1時点の値で
    /// 焼き込む近似になる(裁定174 の oracle は静止 Group だけを要求している)。
    /// 同じ理由で、焼き込みが起きる子は既存のアニメーション(position 等)が
    /// この1 Hold keyframe で潰れる。
    ///
    /// `groups` に Group でない/存在しない id が混じっていても無視する(黙って
    /// 飛ばす — 呼び手が `selected_layers` を Group だけへ絞らずに渡しても安全)。
    /// 返り値は解放された子 id の列(呼び手の選択規則「解除後は旧子ら選択」用)。
    pub fn ungroup_layers(&mut self, groups: &[LayerId]) -> Result<Vec<LayerId>, StoreError> {
        if groups.is_empty() {
            return Ok(Vec::new());
        }

        let t = RationalTime::ZERO;
        let view = self.view();
        let present = view.layers();

        let mut intents = Vec::new();
        let mut released = Vec::new();

        for &group in groups {
            let Some(meta) = view.meta(group)? else {
                continue;
            };
            if meta.source != LayerSource::Group {
                continue;
            }
            // 凍結中の Group は ungroup を理由つき拒否する(裁定119、レーン仕様
            // (c) の論証): ungroup は Group の変換を子ローカルへ焼き込んでから
            // 子の parent を書き換える — これは「凍結中の中身への編集」そのもの
            // (焼き込みは子の position/rotation/scale/skew track を書き換えうる)
            // なので、他の子孫編集 Intent と同じく `check_not_frozen` 相当の扱いを
            // 受けるべきだが、対象は「Group 自身が frozen か」であって「Group の
            // "祖先"が frozen か」ではない(祖先が frozen な場合は、この後ろで
            // 子へ積む `Intent::SetAttrs` が `check_not_frozen` に自然に引っかかって
            // 拒否される — 二重にチェックする必要はない)。**黙って skip しない**
            // (`meta.source != Group` の分岐と違い、これは「対象が正しいのに
            // 凍結されているので今は無理」という積極的な拒否なので、理由つき Err
            // にする — 裁定119「黙って自動解凍しない」)。
            if view.attrs(group)?.unwrap_or_default().frozen {
                return Err(StoreError::Property(format!(
                    "layer {} は凍結中(frozen)なので ungroup できない \
                     (先に unfreeze すること)",
                    group.0
                )));
            }
            let new_parent = view.attrs(group)?.and_then(|attrs| attrs.parent);
            let group_local = view.local_transform(group, t)?;
            let identity = affine2_is_identity(group_local);

            for &child in &present {
                let Some(child_attrs) = view.attrs(child)? else {
                    continue;
                };
                if child_attrs.parent != Some(group) {
                    continue;
                }

                intents.push(Intent::SetAttrs {
                    layer: child,
                    patch: LayerAttrsPatch {
                        parent: Some(new_parent),
                        ..Default::default()
                    },
                });

                if !identity {
                    let anchor = read_vec2(&view, child, crate::property::ANCHOR, [0.0, 0.0], t)?;
                    let child_local = view.local_transform(child, t)?;
                    let baked = bake_child_local(group_local, child_local, anchor);
                    intents.extend(baked.into_intents(child)?);
                }

                released.push(child);
            }

            intents.push(Intent::RemoveLayer(group));
        }

        self.apply_all(intents)?;
        Ok(released)
    }

}

/// G1(裁定174)ungroup が子へ書き戻す、焼き込み後の local transform の5値。
/// [`bake_child_local`] の出力 — `Intent::SetTrack` へ渡す1本ずつの Hold
/// keyframe track を [`Self::into_intents`] が組む。
struct BakedChildTransform {
    position: [f64; 2],
    rotation_degrees: f64,
    scale: [f64; 2],
    skew_degrees: f64,
    /// 常に 0.0 — [`bake_child_local`] の分解は skew axis を正準形(x軸に沿った
    /// shear)へ畳んで返すため(doc 参照)。
    skew_axis_degrees: f64,
}

impl BakedChildTransform {
    fn into_intents(self, layer: LayerId) -> Result<Vec<Intent>, StoreError> {
        Ok(vec![
            Intent::SetTrack {
                layer,
                property: PropertyId::new(crate::property::POSITION)?,
                track: still(Value::Vec2(self.position)),
            },
            Intent::SetTrack {
                layer,
                property: PropertyId::new(crate::property::ROTATION)?,
                track: still(Value::F64(self.rotation_degrees)),
            },
            Intent::SetTrack {
                layer,
                property: PropertyId::new(crate::property::SCALE)?,
                track: still(Value::Vec2(self.scale)),
            },
            Intent::SetTrack {
                layer,
                property: PropertyId::new(crate::property::SKEW)?,
                track: still(Value::F64(self.skew_degrees)),
            },
            Intent::SetTrack {
                layer,
                property: PropertyId::new(crate::property::SKEW_AXIS)?,
                track: still(Value::F64(self.skew_axis_degrees)),
            },
        ])
    }
}

/// t=0 の1本だけを持つ Hold track。焼き込み結果を静止値として書く
/// (`fixture.rs`/`transform_hierarchy.rs` の `still()`/テスト用ヘルパーと同じ形)。
fn still(value: Value) -> KeyframeTrack {
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: RationalTime::ZERO,
        value,
        interp: Interp::Hold,
        spatial: None,
    });
    track
}

/// `group_local`(Group 自身の local `Affine2`)を `child_local`(子の現在の
/// local `Affine2`、anchor 込み)へ乗算し、子の**新しい** local を
/// position/rotation/scale/skew/skew_axis の5値へ分解して返す(裁定174
/// ungroup の焼き込み本体)。
///
/// ## 数式
///
/// `child_local`(H1 正本 `LayerPlacement::from_transform` の形)は
/// `X · T(-anchor)`(`X = T(position)·R(rotation)·Skew·S(scale)`)。子の
/// **新しい** parent は Group 自身の親(呼び出し側が `attrs.parent` を
/// 付け替える)なので、子の新しい world が旧 world と一致するには
/// `new_local = group_local · child_local`(anchor は変えない前提で解くと
/// grandparent の world は両辺で相殺されるので、ここには一切現れない —
/// 祖先が何段あっても同じ式)。
///
/// anchor を変えずに済ませるため、`X`(anchor 適用前の部分)だけを取り出して
/// `X' = group_local · child_local · T(anchor)` を計算し、`X'` を
/// `T(position')·R(rotation')·Skew'·S(scale')` へ分解し直す:
/// - `X'` の並進成分がそのまま `position'`
/// - `X'` の線形成分(2x2)を「回転 × x軸に沿った shear × 非一様 scale」
///   (skew_axis=0 の正準形、`R(θ)·ShearX(skew)·Diag(sx,sy)`)へ QR 的に分解する
///   — 任意の可逆 2x2 行列はこの形に一意分解できる(標準の Gram-Schmidt/QR)。
///   `skew_axis` は常に 0 で返す(元の子が別の axis を使っていても、視覚的に
///   等価な axis=0 表現へ正規化される — **既知の制限**、doc 冒頭参照)。
fn bake_child_local(
    group_local: glam::Affine2,
    child_local: glam::Affine2,
    anchor: [f32; 2],
) -> BakedChildTransform {
    use glam::{Affine2, Mat2, Vec2};

    let x = child_local * Affine2::from_translation(Vec2::new(anchor[0], anchor[1]));
    let x_prime = group_local * x;

    let position = x_prime.translation;
    let linear = x_prime.matrix2;

    let col0 = linear.x_axis;
    let sx = col0.length();
    let theta = if sx > 1e-6 { col0.y.atan2(col0.x) } else { 0.0 };

    let rest = Mat2::from_angle(-theta) * linear;
    let sy = rest.y_axis.y;
    let skew_tan = if sy.abs() > 1e-6 { rest.y_axis.x / sy } else { 0.0 };

    BakedChildTransform {
        position: [position.x as f64, position.y as f64],
        rotation_degrees: theta.to_degrees() as f64,
        scale: [sx as f64, sy as f64],
        skew_degrees: skew_tan.atan().to_degrees() as f64,
        skew_axis_degrees: 0.0,
    }
}

/// `group_local` が恒等に近いか。恒等なら [`Document::ungroup_layers`] は子の
/// transform track を一切書き換えない(`group_layers` が作った直後の Group ——
/// 最も普通のケース — で不要な skew_axis 正規化などの副作用を避けるため)。
fn affine2_is_identity(m: glam::Affine2) -> bool {
    const EPS: f32 = 1e-4;
    m.translation.length() < EPS
        && (m.matrix2.x_axis - glam::Vec2::X).length() < EPS
        && (m.matrix2.y_axis - glam::Vec2::Y).length() < EPS
}

/// `layer` の `name` という property(Vec2)を読む。無ければ `default`。
/// `ungroup_layers` が anchor を読むためだけの薄い口 — `local_placement_transform`
/// 内部の汎用 `vec2` クロージャ([`crate::view`] 側の私法)を複製しない、
/// この1プロパティ専用の最小版。
fn read_vec2(
    view: &StoreView,
    layer: LayerId,
    name: &str,
    default: [f32; 2],
    t: RationalTime,
) -> Result<[f32; 2], StoreError> {
    let property = PropertyId::new(name)?;
    match view.value_at(layer, &property, t)? {
        Some(Value::Vec2(v)) => Ok([v[0] as f32, v[1] as f32]),
        Some(other) => Err(StoreError::Property(format!(
            "{name} に2成分でない値が入っている: {other:?}"
        ))),
        None => Ok(default),
    }
}
