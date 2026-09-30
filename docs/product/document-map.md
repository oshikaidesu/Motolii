# Stage 5 — 文書の救出と照合

この会話で確定した意味と、既存資料に残る根拠を現行入口へ接続した。Git履歴全体を再調査した記録ではない。

| 対象 | 現行の読み先 | 救出元・扱い |
|---|---|---|
| 製品の目的・ユーザー意図優先 | [concept](../concept.md) | 2026-09-05の利用者訂正。[照合資料](../reviews/2026-09-05-concept-alignment.md)。旧「自分用」等の禁止を履歴へ分離 |
| Lottie基幹モデル／Rerunビュー | [concept](../concept.md) | 「RerunにAEのガワ」後の明示訂正。旧「Lottieは審判だけ」と競合させない |
| 同じ空間・直接操作・段の開示 | [根底](../ideal.md) | 設計原則として継承。現在の明示操作を一律禁止する文書にしない |
| Vism / vgpu | [concept](../concept.md)、[既存体系](../../motolii/reference/vgpu-vism.md) | [Vism構想](../vism-package-concept.md)は資料として保持。全配布構想をStage 5の完了条件に追加しない |
| Browser・Inspector・Desk | [UIと操作](product-contract.md) | [Inspectorと机](../reviews/2026-09-02-inspector-and-desk.md)、利用者の四領域・分類保持・色常設の指示 |
| マスク・クリッピング | [クリッピング裁定](../reviews/2026-09-05-timeline-clipping.md) | エフェクトではなくレイヤー関係としての入口 |
| 見た目・グループ・キー・ジェスチャ | [UIと操作](product-contract.md) | 2026-09-05〜06の画像と実操作による訂正。初期の仮実装を採用済みとしない |
| 検証版の持ち戻し | [workspace.json](workspace.json)、[適応helper一覧](imported-edit-helpers.json) | 外部checkpointと元ソースのhash。doc/renderの正本は本体一つ |
| 技術移行・開発入口・未完 | [Stage 5](README.md) | Dioxus/Blitzの通常路をFlutterへ変更。旧コードは回帰参照 |
| 候補・未採用案 | [パネルレビュー候補](../reviews/2026-09-05-panel-review-candidates.md) | 候補の存在を仕様承認と見なさない |

## 追加で救出した境界

[保存・描画・検証](technical-boundaries.md)に、.rrdとLottieモデルの区別、共有GPU経路、hot reload測定の条件、比較版の実データ検証を接続した。旧ui-visual-language／ui-interaction-language／ui-quality-bar／pitfalls-and-roadmapには世代を明記し、現行の入口へ誘導した。

## 宿題として残すもの

Browser分類の詳細、型付きbridgeとUI責任分離、一般の親空間補償、3Dギズモ全機能、UI sessionの永続化、他OS・配布bundle。新しい機能禁止ではなく、現時点の未完として扱う。

## 漏れを再び作らない入口

`workspace.json`の`documents`が、コンセプト・操作・移行・照合の必須文書を指す。`scripts/motolii-ui.sh check`で存在とローカルリンクを確認する。製品意味はconcept／product-contract、実装状況はStage 5／workspaceへ置き、二つを混ぜない。

## 2026-09-06 最終入口レビュー

初見貢献者・UI実装担当・保守担当の3視点で、各5〜8回以内の軽いread-only確認を実施。歴史全体や実装の深い評価ではない。

- root Cargoが欠損旧workspaceを指す問題を解消。既定はFlutter native、targetは既存cacheを継続。旧manifest/lockはhistoryに保存。
- 現役CIの旧app／個人checkout依存を除去。Stage 5構成と文書の確認へ切り替えた。CIのリモート実行結果は未確認。
- PR templateの未定義top seatと、Cargo冒頭の旧hotpatch tip説明を修正。
- 再読で入口の指摘が解消したことを確認。構成check・文書全体check・root Cargo metadata(--locked --no-deps)はローカル通過。

入口レビュー後に[モジュール分離](modules.md)を実施。型付きpayload/snapshotとpanel内部の詳細整理は未完であり、入口の合格と混同しない。
