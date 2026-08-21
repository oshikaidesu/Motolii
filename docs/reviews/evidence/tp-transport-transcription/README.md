# TP: transport 転写 — 証拠(2026-08-22)

レーン: TP(レーンボード)。対象: ψ 転写ギャップ台帳 #23/#24(チグハグ知覚 主因1位)・
裁定172 §1「transport 系は M4 後の転写レーンへ」。write-set=
`next/shell/motolii-shell/src/lib.rs`(`transport`/`transport_slider_style`)+
`next/reference/mocks/timeline-semantics.html`。

## before/after(実窓 — `screencapture`、`--fixture` 起動の実ウィンドウ、Retina 2x)

`--fixture --screenshot` 器具は transport を**帯色のみ**で描く(`screenshot.rs`
コード注記「timecode は文字なので描かない」)ため、slider/frame カウンタの意匠差分
は器具では撮れない(下記「器具の対象外確認」参照)。そのため実ウィンドウを
`--fixture` で起動し `screencapture -R` で撮った。

| file | 何の証拠か |
|---|---|
| `before-full.png` | 変更前の実ウィンドウ全体(2048×1592)。transport 帯に iced 既定の太トラック+丸い金色つまみ、frame カウンタが ACCENT 色 |
| `after-full.png` | 変更後の実ウィンドウ全体。同一 fixture Document・同一 playhead(frame 900) |
| `before-transport-strip.png` / `after-transport-strip.png` | 下端160px(transport 帯+status 帯)の切り出し。並べて見るための対 |
| `after-handle-zoom.png` | つまみ部分を3倍最近傍拡大。角丸ゼロの縦長方形であることの確認 |

### 差分は transport 帯だけ(それ以外バイト不変)

`before-full.png`/`after-full.png` を numpy で画素差分(閾値 |Δ|>10 でアンチエイリアス
ノイズを除外):

```
num differing pixels (>10): 16167 / 3260416
y range: 1475 1506   (画像高さ 1592。h-160=1432 以降が transport 帯)
differing rows ABOVE transport band: 0
```

Inspector・Stage・Timeline・status 帯を含む transport 帯より上は完全に不変
(閾値以下のドット単位の差もゼロ行)。

## 器具(`--fixture --screenshot`)の対象外確認

同一 before/after ソースを `--fixture --screenshot <path>` で撮った
`instrument-before.png`/`instrument-after.png` は **バイト完全一致**(`cmp` で確認)。
transport は器具側で帯色のみの再現(`screenshot.rs:24,1027` の doc 注記どおり)
のため、この器具は今回の変更を検分できない — **器具対象外**として記録する
(ORACLE の想定どおり)。

## 再現手順

```sh
# before: 変更を stash → build → 実行 → screencapture(ウィンドウ位置は
# `osascript -e 'tell application "System Events" to get position/size of window 1
# of (first process whose unix id is <pid>)'` で取得)
git stash push -- next/shell/motolii-shell/src/lib.rs
cargo build -p motolii-shell -j 4
./target/debug/motolii-shell --fixture &
screencapture -x -R<x>,<y>,<w>,<h> before-full.png
kill %1
git stash pop

# after
cargo build -p motolii-shell -j 4
./target/debug/motolii-shell --fixture &
screencapture -x -R<x>,<y>,<w>,<h> after-full.png
kill %1

# 器具側(参考・対象外の確認用)
./target/debug/motolii-shell --fixture --screenshot instrument-before.png  # stash 中
./target/debug/motolii-shell --fixture --screenshot instrument-after.png  # stash pop 後
cmp instrument-before.png instrument-after.png  # IDENTICAL
```
