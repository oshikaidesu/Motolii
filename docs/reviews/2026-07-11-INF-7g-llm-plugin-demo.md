# INF-7g: LLMプラグイン実演記録(2026-07-11)

ステータス: **歴史的実演完了**（2026-07-11時点で人間レビュー差し戻し0回、機械判定のみで緑）。これはscaffoldと自動審判が一例の往復を減らした証拠であり、現行の実装レビュー省略、第三者runtime、Vism配布の成立を意味しない。現在の位置づけは[Unit 9E歴史回収](2026-07-23-historical-llm-plugin-demo-lineage-recovery.md)を参照。

INF-7a〜f が揃った後の証拠: エージェントが規約準拠のプラグイン1個を書き、`cargo test` 緑まで通した。

## プロンプト(要約)

> INF-7g: `scripts/new-plugin.sh` 型紙に沿い Filter 1個を参照実装へ追加せよ。完了条件は `validate_node_desc`・純関数検査・ゴールデンが緑。人間レビュー差し戻しなし。

## 成果物

| 項目 | 内容 |
|---|---|
| プラグイン | `core.filter.opacity` (`OpacityFilter`) |
| 意図 | premul RGBA に `amount`(0..1) を乗算 |
| 生成起点 | INF-7e スケルトン規約(必須メタ+アリティ) |
| GPU | wgpu/WGSL のみ。`PipelineCache` ホスト所有(F-10) |

## 機械判定(差し戻し=CIのみ)

| チェック | コマンド/経路 | 結果 |
|---|---|---|
| NodeDesc / レジストリ | `cargo test -p motolii-plugin` | 緑(Filter数=3) |
| ベンダーAPI deny / panic | conformance | 緑 |
| 純関数 | `cargo test -p motolii-testkit --test purity` (`opacity_filter_is_pure`) | 緑 |
| ゴールデン | `cargo test -p motolii-testkit --test opacity_filter` | 緑 |

**人間差し戻し回数: 0**(INF-7 の目的どおり、目視チェックリストは回していない)。

## 学び

- 初期状態を `new-plugin` / 参照 Tint 型紙に固定すると、desc 欠落やアリティ違反で止まらない
- 純関数ヘルパー(INF-7f)があるため「動いた気がする」で終わらず、隠れ状態の負例と対で固定できる

## 現在の停止線（2026-07-23追補）

- 当時のFilter数、reference registry配置、Tint型紙は歴史的コード事実であり、現行構成の正本ではない。Opacityは現在、公開façadeだけを使う外部first-party crateへ移っている。
- `scripts/new-plugin.sh`は現行in-tree参照実装の入口であり、第三者向けpackage／install／loader／公開scaffoldの完成を証明しない。外部作者scaffoldはVSM-A4の課題である。
- 「差し戻し0回」は一回の実演結果であり、LLM生成差分の人間／Codexレビューを不要とする運用規則ではない。
- conformance、purity、goldenが緑でも、provenance、trust、permission、sandbox、配布、互換責任は別に審判する。
