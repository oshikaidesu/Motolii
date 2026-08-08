# gizmo機構 — 既知実装採択preflight

日付: 2026-08-08
状態: **調査完了 / 採択裁定は未了 / 実装発注は不可**

## 0. 扱い

`AGENTS.md`「既知実装優先 — 新設前に探索・採択する」が要求するpreflightを記録する。
**`BUILD JUSTIFICATION`が確定しておらず、本文書だけでgizmo実装を発注できない。**

`known-implementation-adoption-model.md`は「picking／gizmo／bounds を機構classとして調べる」と
指定していたが、調査recordが存在しなかった。本文書がその欠落を埋める。

## MECHANISM CLASS

単一active cameraで投影された2D出力面の上に、選択中objectの
transform操作handle（translate / rotate / scale、軸拘束、pivot、snap補助）を提示し、
pointer gestureをterminal property intentへ変換する機構。

**Motolii固有の制約**（すべて既決）:

- 単一active camera / 単一active binding（複数view同時なし）
- **depth遮蔽はv1範囲外**（`concept.md`「レイヤ間で深度による相互遮蔽を厳密にやりたい場合はv1の範囲を超える」）
- レイヤ側は「3D素材としての配置 / 2D-ishなtransform」だけを持つ
- hit-test / gestureはRust headless、確定はD2、描画はcanonical出力外のoverlay
- 正準座標: 原点中央・Y-up・高さ1.0

## KNOWN IMPLEMENTATION SEARCH

2026-08-08、空workspace・repo tool 0・`WebSearch`/`WebFetch`限定のfresh runで実施。
一次資料（crates.io API / GitHub / docs.rs / 各製品公式マニュアル）のみを採用。
別family（GPT-5.6 Sol）による反例監査を併走。

## CANDIDATES

| 候補 | license | 描画前提 | camera受け取り | depth遮蔽 |
|---|---|---|---|---|
| **`transform-gizmo`** | **MIT OR Apache-2.0** | **framework非依存core**。`Gizmo::draw`は**viewport座標系の頂点データを返すのみ**で描画は利用側 | **`GizmoConfig`に`view_matrix`/`projection_matrix`。外部から渡す** | configにdepth関連field**なし**（不在。一次資料に明記なし＝**不明**） |
| `transform-gizmo-egui` | MIT OR Apache-2.0 | `GizmoDrawData`を返しegui側で描画する**2D overlay方式** | 同上 | 明記なし・不明 |
| `transform-gizmo-bevy` | MIT OR Apache-2.0 | Bevy 3D sceneへ統合するplugin | `GizmoCamera` component経由 | 明記なし・不明 |
| `bevy_transform_gizmo` | 要再確認 | Bevy統合 | — | **「Gizmo always renders on top of the main render pass」と明記** |
| `egui-gizmo` 0.16.2 | **単一MIT**（LICENSE本文確認） | egui統合 | — | 明記なし |

`transform-gizmo`の`GizmoConfig`は
`modes` / `snapping` / `snap_angle` / `snap_distance` / `snap_scale` /
`orientation`(global/local) / `pivot_point` を持つ。
数学型はmint経由でnalgebra/glam/cgmathと相互運用可能、GUI/engineへの直接依存なし。

`egui-gizmo`は`transform-gizmo`リポジトリへ統合済みで**独立開発は停止**している
（最終タグ`egui-gizmo-0.16.2`、2024-03-14）。

## 製品先例（PATTERN）

9製品（Blender / AE / Cavalry / Figma / Unity / Unreal / Cinema 4D / Houdini / Godot）を
公式マニュアルで調査した。

**軸「背後に回ったhandleの遮蔽」について、一次資料で明確な仕様記述が取れた製品はゼロ。**

- Houdini: 「handleがgeometryを覆い隠す場合は脇へ移動できる」→ **手前描画を示唆（間接）**
- Unity: `Handles.zTest`既定が`Always`。ただし**カスタム拡張API**であり組み込みツールの仕様ではない
- Unreal: フォーラム投稿のみ → **一次資料でないため不採用**
- Blender: `Don't write into the depth buffer`プロパティの存在は検索結果で確認できたが、
  当該APIページが403で**直接検証できず**
- 他5製品: **不明**

Blenderは GPL のため`references.md`により`PATTERN`限定（コード流用不可）。

## REJECTED CANDIDATES

現時点で明示的に棄却した候補は無い。**採択裁定自体が未了である。**

## ADOPTION ROUTE（案・未裁定）

`transform-gizmo`（core）が次を満たす:

- 描画先を選ばせない（頂点データのみ返す）→ rust-skia overlayでもwgpuでも成立
- `view_matrix`/`projection_matrix`を外部から受ける → Motoliiの単一active cameraと
  Observation Contractの「Hostが配る評価済み観測を消費する」形と一致
- depth概念を持たない → `concept.md`の「depth遮蔽はv1範囲外」と一致

**ただし裁定はしていない。** 下記の未確認事項が残る。

## 未確認事項（裁定を妨げるもの）

1. **`transform-gizmo`のdepth挙動は一次資料で未確認。**
   「configにfieldが無い」は事実だが「常に手前に描く」は**推論**である。
   当初主担当がこれを事実として扱ったが、反例監査により**推論へ格下げした**
2. **bus factor未評価。** `transform-gizmo`が単一作者/単一repoに依存するリスクを
   反例監査が「URL取得制限により**判定不能**」とした。
   `egui-gizmo`が同repoへ統合された事実は集約の傍証だが、評価そのものではない
3. **製品先例の「不明」7件が本当に不明か未確認。** 反例監査も判定不能とした
4. `bevy_transform_gizmo`のlicenseを一次資料で未確認

## THIN MOTOLII SEAM（案・未裁定）

Motolii固有として残るのは、canonical座標↔gizmo座標の変換、
hit結果からD2 terminal intentへの写像、`R2-SELECTION-AUTHORITY`との接続、
描画のrust-skia overlayへの委譲。**gizmo frameworkは書かない。**

## BUILD JUSTIFICATION

**`NONE`と断定できない。** 未確認事項1〜4が残るため、
「既存実装で足りるか」の判定が完了していない。

`AGENTS.md`は「`BUILD JUSTIFICATION`が`NONE`以外なら通常発注を止め、利用者例外へ戻す」と定める。
したがって**本文書の状態ではgizmo実装を発注できない。**

## BUILD: FORBIDDEN

gizmo機構そのものの新規実装は禁止する。上記候補の採択裁定が先である。

## 次に必要なこと

1. 未確認事項1〜4を閉じる（特に1と2）
2. `N-OVERLAY`（rust-skia移管）が成立してから接続先を確定する
   — 描画先が無い状態でgizmoを採択しても置き場所が無い
3. 採択裁定を`decision-index`へ登録する

## 非目標

- 本文書を根拠にgizmo実装を発注すること
- 未確認事項を推論で埋めて裁定すること
- Blenderのコードを参照すること（GPL、`PATTERN`限定）
- gizmo frameworkをMotolii内に新設すること
