# ime-acceptance (M3 実装ガード1 / Issue #56)

Slint `TextInput` で日本語 IME 受け入れチェックリスト 4 項目を**人手審判**するスパイク骨格。
製品 `Document`/schema には触れない。`spikes/` 完結。

**状態: 骨格 landed / 実機審判 pending** — クラウド CI では GUI 実走不可。合否は開発主機で記録する。

## チェックリスト (仕様どおり)

| # | 項目 | 手順 |
|---|---|---|
| 1 | preedit下線表示 | ローマ字入力→変換前の未確定表示 |
| 2 | 候補ウィンドウ追従 | カーソル移動で候補が追従するか |
| 3 | Enter未食い | 未確定のまま Enter → 下部ショートカットログに出なければ合格 |
| 4 | 長文歌詞連続入力 | `LONG_LYRIC_SAMPLE` を貼付/連続入力 |

対象: Windows MS-IME / macOS / Linux (fcitx5+Wayland, ibus+X11)

## 実行

```bash
cd spikes/ime-acceptance
cargo run                                    # GUI (開発主機)
cargo test                                   # 静的検査 (ヘッドレス可)
IME_ACCEPTANCE_MANIFEST=../../docs/spikes/ime-acceptance-evidence/manifest.json cargo run
```

## 自動化 (CI / ヘッドレス)

| 検査 | 内容 |
|---|---|
| `cargo build` | スパイクがコンパイルできる |
| `set_ime_allowed` 静的検査 | `slint` / `winit` 依存ソースに API 呼び出しがあるか grep |
| チェックリストハーネス | 4 項目定義・マニフェスト形式・pending 初期状態 |

## 合否記録

`docs/spikes/ime-acceptance.md` (INF-1 形式参考。本スパイクのラベルは M3-GUARD-1)
