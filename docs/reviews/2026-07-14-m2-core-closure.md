# M2コア締結宣言(2026-07-14)

ステータス: **撤回**(2026-07-14)。P1修復(#153/#154)は完了。再宣言という単独の出口は廃止し、[M2基盤再締結ゲート](2026-07-15-m2-foundation-reclosure-gate.md)のA〜C証跡を別PRで満たした場合だけ解除する

## 撤回理由

実査で締結を止める P1 が2件見つかった(CI未検出・#152はレビュー0件のままマージされていた):

1. **D4-FU**: 終端 flush が期待デバイス尺を超える無音を実供給し、Transport 時計が元素材尺を超える → **修復済み** #153 / #147 クローズ
2. **D6**: `doc.layer_source.rect` を plugin_id だけで degraded 除外し、未来版 effect_version の書き出し拒否を迂回 → **修復済み** #154 / #133

D7 は問題なし。D5(#144) は**発注可**(D3+D4+D4-FU充足)。

## 閉じたもの(撤回前の記録・再確認待ち)

M2のコア(Document意味・Undo・参照・可搬性・評価順)を、main到達の自動テスト証跡付きで締結する。これは「フェーズ名のカレンダー完了」ではなく、**証明済み契約の内部宣言**である(凍結ゲート宣言と同型)。

| 面 | 証跡(main) | 代表PR/Issue |
|---|---|---|
| Document意味(スキーマ・validate・PathOp・LookAt等) | `motolii-doc` スキーマ/意味論ゴールデン / D1i系 | #100/#128/#139/#141 ほか |
| Undo | `d2_command` apply↔revert / gesture merge | #109/#130 |
| 参照・可搬性(AssetId・未知plugin保持・migration) | D1f / D1e / D1c-FU | #126/#140/#101 |
| 評価順(F-3) | `d3_eval_order` | #110/#136 |
| クリッピングマスク | `d7_clipping_mask` / `mask_node`(provisional台帳) | #145/#149 |
| 音声基盤+書き出しmux | D4 / D4-FU / D6(`export_refuses_degraded_plugins`) | #129/#151/#150 |

Wave4マージ順: **#151 → #150 → #149**(CI緑確認後)。

## 残チケット(M2内・次発注)

| ID | 内容 | 状態 |
|---|---|---|
| D5(#144) | Transport(音声クロック常時主+DRS)。方針【採択】済み | **発注可**(D3+D4+D4-FU充足) |

D5はB-1音声の再生ヘッド実装でありコア契約の延長だが、本宣言の「Document意味・Undo・参照・可搬性・評価順」閉集合には含めない。M3-U1はD5依存のまま([M3/M4ゲート台帳](2026-07-12-M3-M4-gate-ledger.md))。

## M2 blocker ではないもの(PP-Gate)

次は **M3前の追跡項目(PP-Gate)** とし、本締結をブロックしない:

- Param Pipeline
- Element Domain
- Constraint Graph

これらはDocument恒久面への追加発明を伴い得るため、M2コア締結後・M3入場前に別ゲート/台帳で扱う(本宣言では席のみ記録。仕様本文は別PR)。

## 締結時の検証コマンド(2026-07-14実行・全緑)

```text
./scripts/check-golden-update-policy.sh          # exit 0
cargo test -p motolii-doc --test d1f_unknown_plugin
cargo test -p motolii-export --test d6_audio_mux   # export_refuses_degraded_plugins 含む
cargo test -p motolii-doc --test d2_command
cargo test -p motolii-doc --test d3_eval_order
cargo test -p motolii-doc --test d7_clipping_mask
cargo test -p motolii-nodes --test mask_node
cargo test --workspace
```

main tip at declaration drafting: `e816a93`(Merge #149)。

## 参照

- [M2仕様](../specs/M2-document-model.md)
- [Transport先例調査【採択】](2026-07-14-d5-transport-prior-art.md)
- [M2入場条件](2026-07-11-M2-entry-gate.md)
- [恒久焼き込み予防](2026-07-12-m2-permanence-prevention.md)
