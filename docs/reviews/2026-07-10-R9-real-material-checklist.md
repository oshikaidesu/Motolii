# R9 実素材検証チェックリスト (T11)

日付: 2026-07-10  
対象: M1最終チケット。**機械判定 + 主観判定**の両方が必要。

**判定: 完了** (2026-07-10、人間サインオフ)

## 前提

- 開発主機(macOS想定)に ffmpeg / GPU(wgpu) あり
- `scripts/setup-local-deps.sh` 済みなら `source .tools/env.sh`
- 実素材はリポジトリにコミットしない(ローカルパスのみ)

## 1. プロジェクトJSONを用意

`docs/samples/r9-project.template.json` をコピーし、`input`/`output` を実素材向けに編集。

推奨:

- `"qp0": true` — B-4比較の誤差を抑える(QuickTime再生は `qp0: false` / yuv420p)
- `"frame_count"` — まず数秒(例: 90 @ 30fps)に絞る
- パスは **JSONファイルからの相対パス**

## 2. 自動: 書き出し + B-4一致 + **GUIプレビュー**(既定)

```bash
chmod +x scripts/r9-verify.sh scripts/r9-smoke.sh
./scripts/r9-verify.sh /path/to/your/project.json
```

スモーク(合成動画で手元確認):

```bash
./scripts/r9-smoke.sh
```

GUIを出さない場合のみ `--no-preview`。

- [x] 書き出しmp4をフルスクリーン/実解像度で視聴し、MV素材として使える
- [x] 色・オーバーレイ位置が意図通り(回転素材なら向きも)
- [x] プレビューと書き出しで目視でも破綻がない(B-4の補強)

## 3. 実施記録(2026-07-10)

| 素材 | 解像度 | 備考 |
|---|---|---|
| 4K stock (`1118618_4k_End_3840x2160.mp4`) | 3840×2160 | `qp0: false`、QuickTime再生OK |
| MV素材 (`プロジェクト名 88.mp4` 先頭3秒) | 1920×1080 | 半透明オーバーレイ(alpha=0.5)確認 |

所見:

- 640×360スモーク素材は解像度が低く、実寸品質判断には不向き(意図通り)
- `qp0: true`(yuv444p)はQuickTime非互換 — 主観確認は `qp0: false` で実施
- GUIプレビュー(`r9-preview`)と書き出しmp4の見た目に破綻なし

## 4. 台帳更新

- [x] `docs/specs/M1-vertical-slice.md` の R9 / T11 を完了に更新
- [x] 凍結ゲート入場前チェック完了 → 残件は[2026-07-10-freeze-gate-remaining.md](2026-07-10-freeze-gate-remaining.md)(FG-C1〜C6)

## トラブルシュート

| 症状 | 対処 |
|---|---|
| `max_diff` が tolerance 超え | `qp0: true` を確認。半透明オーバーレイは H.264 往復で誤差が乗るため `--tolerance 16`〜`24` を試す。パススルー検証なら `color` の alpha=0 |
| QuickTimeで開けない | `qp0: true` は yuv444p — `qp0: false` で書き出し、または VLC/ffplay |
| `output file missing` | `--export` 付きで実行、または先に `export-project` |
| プレビューが真っ黒 | 入力パス・`start_frame` を確認。ターミナルに GPU/adapter ログあり |
| CIではR9を回さない | 意図的。合成素材の `motolii-cli/tests/r9_b4_verify.rs` のみCI |
