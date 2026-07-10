# INF-7g: LLMプラグイン実演記録(2026-07-11)

ステータス: **完了**(人間レビュー差し戻し 0 回。機械判定のみで緑)

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
