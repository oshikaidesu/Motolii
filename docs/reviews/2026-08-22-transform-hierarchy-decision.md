# 裁定173 — 変換階層の採択: parent 単一真実+Group マーカー・合成は単一再帰・旧世界移植

日付: 2026-08-22(夜間自律運転) / 状態: **決定** / 起点: 利用者裁定「グループ/親子/シェイプ階層をそろそろ — 再帰的に決まる」+H-survey(`2026-08-22-transform-hierarchy-seam-survey.md`)

## 1. 採択

- **schema = 案(c)**: 既存 `LayerAttrs.parent: Option<LayerId>`(循環ガード実装済み・resolve 未読)を**変換階層の唯一の真実**とし、グループは `LayerSource::Group` の**マーカー variant**として追加する。**Group に members 列は持たせない**(親参照との二重帳簿を構造的に排除 — 2026-08-20 起草の「member 列」表現はこの点で本裁定が上書き。起草文書との突き合わせは H1 発注書に明記)
- **アルゴリズム = 単一再帰**(利用者仮説の採択): キーフレームは各ノードのローカル値のまま、**合成だけが再帰**。`StoreView::resolve_with_solo`(view.rs:822 — 現在ローカル Affine2 のみ)へ親合成を挿入する。compositor は comp 空間の不透明 affine を受けるだけ(4呼び出し点実測)なので **engine/compositor は無改修**
- **スキーマ上は辺が2種**(調査の反例を受理): Group 所属(構造)と parent 参照(変換)は概念として別の辺 — ただし変換の実体は parent 1本に集約されるため、評価アルゴリズムは単一で済む。「単一再帰木」は**アルゴリズム階層で真・スキーマ階層で偽**、と精密化して採択
- **移植>スクラッチ**: 旧世界 `spatial_resolve.rs`(メモ化・循環安全の world-affine 解決)と `timeline_rows.rs`(fold 軸独立の flatten-per-frame 行アルゴリズム・テスト付き)は 2026-08-20 リセット文書が移植予約済みのまま未移植 — **H 束の移植元として正式指定**

## 2. 切片(H 束 — 調査の4切片案を採用・並走条件つき)

| 切片 | 中身 | 前提 |
|---|---|---|
| H1 | store: resolve への親合成(単一再帰・メモ化は旧世界移植)+`LayerSource::Group` marker+circular 拒否の oracle | **MK2 着地後**(store write-set 衝突) |
| H2 | Timeline ツリー行(インデント・fold — 旧世界 timeline_rows 移植)+rail の親子表示 | **TL-P1 着地後**(timeline-pane 衝突) |
| H3 | 親選択 UI(pick 系 — normal-map の parenting 行消化)+Inspector の parent 表示 | I-tokens 着地後 |
| H4 | シェイプ内階層(motolii-vector の Shape 入れ子 — 現状 flat)— 同じ合成関数を使う | H1 着地後 |

- 旧世界の `LookAt/Follow`(プロパティ拘束)は**今回のスコープ外**(将来玉として台帳へ — 変換木とは別種の辺)
- isolate/freeze(2026-08-20 起草の残り2分解)は H 束の後の別裁定

## 3. oracle の型(全切片共通)

親移動→子の最終位置の数値証明・循環 Intent の拒否(既存ガードの oracle 化)・undo 粒= 1 gesture 1 undo・serde 後方互換(parent 無し旧 Document)・flatten 行アルゴリズムは旧世界テストの移植で赤→緑
