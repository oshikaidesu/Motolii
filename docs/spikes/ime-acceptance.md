# 日本語 IME 受け入れスパイク結果 (M3 実装ガード1)

作成日: 2026-07-15 / Issue [#56](https://github.com/oshikaidesu/Motolii/issues/56)

## 結論 (M3 実装ガード1 / Issue #56 **未実走 — 実機審判 pending**)

記録形式は [s1-slint.md](s1-slint.md)(INF-1) を参考にした。**本スパイクのラベルは INF-1 ではない** (M3-GUARD-1)。

**骨格は landed** (`spikes/ime-acceptance/`)。クラウドエージェントは GUI 実機が無いため、チェックリスト 4 項目の合否は**開発主機での実走待ち**。本ドキュメントは合格と記録しない。

| 層 | 内容 | 状態 | 証拠 |
|---|---|---|---|
| 0 骨格 | Slint `TextInput` + チェックリスト UI + マニフェスト形式 | **landed** | `spikes/ime-acceptance/` |
| 1 自動化 | `cargo build` / `set_ime_allowed` 静的検査 / 4 項目ハーネス | **CI可** (下表) | `cargo test` in `spikes/ime-acceptance/` |
| 2 実機審判 | チェックリスト 4 項目 × 対象 OS/IME | **pending** | (未実走) |
| 3 総合 | M3 ガード1 合否 | **pending** | 本ドキュメント更新待ち |

## チェックリスト (仕様どおり)

[M3-ui-integration.md 実装ガード1](../specs/M3-ui-integration.md) から転記。1 つでも落ちたらテキスト入力だけ別経路の設計余地を確保する。

| # | 項目 | 判定 | 備考 |
|---|---|---|---|
| 1 | preedit下線表示 | **pending** | |
| 2 | 候補ウィンドウがカーソル位置に追従 | **pending** | |
| 3 | 変換中の Enter がアプリのショートカットに食われない | **pending** | スパイク下部のショートカットログで確認 |
| 4 | 長文歌詞の連続入力 | **pending** | `LONG_LYRIC_SAMPLE` 参照 |

### 対象プラットフォーム (各で上表を埋める)

| プラットフォーム | IME | 表示サーバ | 総合 | 記録日 |
|---|---|---|---|---|
| Windows | MS-IME | — | pending | |
| macOS | 標準 IME | — | pending | |
| Linux | fcitx5 | Wayland | pending | |
| Linux | ibus | X11 | pending | |

## 自動化 (2026-07-15 骨格コミット時点)

クラウドで実行可能な検査のみ。**IME 合否とは別** — 下表は静的検査の成否。

| 検査 | コマンド | 期待 | 状態 |
|---|---|---|---|
| ビルド | `cd spikes/ime-acceptance && cargo build` | 成功 | **pass** (2026-07-15 クラウド) |
| TextInput 使用 | `cargo test spike_source_uses_text_input` | `TextInput` in main.rs | **pass** |
| `set_ime_allowed` 静的検査 | `cargo test winit_or_slint_sources_reference_set_ime_allowed` | winit/slint ソースに呼び出し | **pass** |
| 4 項目ハーネス | `cargo test checklist_has_four_items` 等 | 全 pending テンプレ | **pass** |

再現:

```bash
cd spikes/ime-acceptance
cargo test
# 2026-07-15: 6 passed, 1 ignored (record_manual_template)
```

## 実機審判手順 (開発主機)

```bash
cd spikes/ime-acceptance
cargo run
# 任意: スケルトンマニフェスト出力
IME_ACCEPTANCE_MANIFEST=../../docs/spikes/ime-acceptance-evidence/manifest.json cargo run
```

1. 各チェックリスト項目を上表の手順どおり確認
2. プラットフォーム表を pass/fail で更新
3. スクリーンショットがあれば `docs/spikes/ime-acceptance-evidence/` に保存
4. 本ドキュメントの「結論」節を **合格** または **不合格** に更新 (全対象で pass のときのみ合格)

手動テンプレ出力: `cargo test -- --ignored record_manual_template`

## 不合格時の代替案 (参考・現時点は不適用)

仕様どおり、審判前に記録のみ:

| 代替 | 内容 |
|---|---|
| A | テキスト入力フィールドだけ WebView/別ウィジェットに切り出し、レンダ/UI シェルは Slint のまま |
| B | 歌詞・長文専用の外部エディタ連携 (確定後に Document へコミット) |
| C | winit 層で `set_ime_allowed` / IME 位置の明示制御をホスト側に追加 |

## 検証コード

- `spikes/ime-acceptance/` — README 参照
- 証拠 (実走後): `docs/spikes/ime-acceptance-evidence/` (未作成)

## 関連

- [M3-ui-integration.md 実装ガード1](../specs/M3-ui-integration.md)
- [s1-slint.md](s1-slint.md) (INF-1 — 基盤採否は別途完了。深い IME は本スパイク)
- Issue [#56](https://github.com/oshikaidesu/Motolii/issues/56)
