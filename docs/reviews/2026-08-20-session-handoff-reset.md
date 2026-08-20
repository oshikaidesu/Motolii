# セッション引き継ぎ — リセット初日

日付: 2026-08-20
状態: **引き継ぎ**
セッション ID: `9682b064-b8c7-4dae-936e-2d2e66647027`
ブランチ: `claude/motolii-reset-redesign-1a8b01`(worktree `.claude/worktrees/motolii-reset-redesign-1a8b01`)
最終コミット: `916e3673`(この日の commit 69本)

**バイアスを抜いて書く。** 進んだことより、**信じてはいけないこと**を先に書く。

---

## 1. まず疑うべきこと

| # | 事実 | なぜ疑うべきか |
|---|---|---|
| 1 | **実機で1度も動かしていない** | `motolii-shell` は `cargo build` が通り運転席6本が green だが、**窓を開けて人が触った回数はゼロ**。旧 workspace の教訓(「良い判定は全て機械検証止まりで実機の手触りは人間未検証」)が**そのまま再現している** |
| 2 | **`--release` で走らせていた期間、`debug_assert` が全部黙っていた** | `Document::load` の store 同一性のずれを見逃していた(裁定103)。**他にも同種の見逃しがある可能性がある**。debug で通したのは最後の3コミットだけ |
| 3 | 地図の「採用予定 163→152」は**判断の数**であって実装量ではない | 1行の重さが全く違う。`text-style-feature/tag` と `trim-path/ty` が同じ1行 |
| 4 | 裁定106本のうち、**実測に裏付けられているのは R0/R1/R2 と各束の試験だけ** | 残りは設計判断で、多くは「先例がこう言っている」に依拠している。先例の読み違いは起こりうる |
| 5 | `owns:` 7,386行という数字は **crate 単位の粗い集計** | `check.sh` は crate の根の marker しか見ないので、`wraps:` を名乗る crate の中に自前実装が入っても検出しない(裁定34 で1度実際に起きた) |
| 6 | **旧 workspace は無傷**で、既定 bin も旧 egui shell のまま | 新側が実用になっていないので触っていない。**「移行した」と読まないこと** |

---

## 2. 何が起きたか(1行ずつ)

- ドリフトの累積をリセットし、軸を1本にする裁定が出た(`2026-08-20-reset-to-one-axis.md`)
- 新 workspace `next/` を独立 cargo workspace として建てた。旧は歴史証拠として残す
- R0 probe で「rerun store が編集に耐えるか」を実測(6/6)。**訂正1件** — `LatestAtQuery` は単一 timeline しか取らないので Document は `comp` 軸に載らない
- store → eval → compositor → export の鎖が閉じた。**Preview = Export を現物で検証**(旧 workspace が最後まで未検証だった項目)
- iced shell の骨が立ち、**iced fork が不要になった**(Stage を CPU 経路にしたので seam 2 が要らない)
- 利用者裁定で**「保守をしたくない」を軸に格上げ**。未使用の移植資産 1,256行 + 約330行を落とした
- **Lottie 公式スキーマの全語彙を地図にして 486項目を判断**(未判定0)。不採用が6割
- Rive の text defs 73行を追加。**地図が黙っていた穴**(line-height/tracking の正本が3つ)が出た
- **`shape-1` 束を並列エージェントが完了**(38/38、worktree 隔離)。同時に私のバグを1つ発見

---

## 3. 今の形

```
next/
  core/    motolii-core   owns  有理数時刻・frame・LayerPlacement(affine)
           motolii-eval   owns  keyframe 補間・bezier・Value
           motolii-store  owns  Document の意味・保存読込
           motolii-testkit owns 外部ツール欠落時の方針
  engine/  motolii-compositor wraps  re_renderer
           motolii-engine     wraps  **1フレームを出す唯一の経路**
           motolii-media      owns   decode/encode/mux
           motolii-export     wraps  回して書いて報告するだけ
           motolii-vector     owns   パス演算子 + tiny-skia
  shell/   motolii-shell      wraps  iced。store 投影のみ
  probes/  r0 / r1 / r2
```

- **debug で 179 tests green**
- 自前実装(保守の負債)**7,386行**
- **fork は rerun 1本のみ**。依存グラフに egui / eframe は 0件
- 裁定 **106本**(`next/DECISIONS.md`)

---

## 4. 規律(これを壊さないこと)

1. **marker** — 各 crate の根が `//! wraps:` か `//! owns:` で始まる。`owns:` は「上流に無い」という主張で、そこだけがレビュー対象
2. **地図** — `next/reference/lottie-coverage.tsv` が全語彙 557項目。`status` / `note` / `evidence` / `unit` / `source` の5列で管理し、**6本の試験が2方向照合する**
3. **evidence** — `採用済` の行は**コード中に実在する識別子**を持つ。試験が grep するので自己申告にならない
4. **unit** — `採用予定` の行が属する発注単位。**完了条件 = 束の行が全部 採用済 + evidence 実在**
5. **`./check.sh`** — marker の書き忘れ / `owns:` 総量 / 未判定の数 / 発注単位の残り、を毎回出す
6. **台帳を増やさない** — backlog も roadmap も作らない。全部地図の列にする

---

## 5. 残っている発注単位

| 束 | 残り | 依存・注意 |
|---|---|---|
| `text` | 75 | **裁定98 の読み直しが未反映**(下記6) |
| `shape-2` | 27 | 旧 `pathgeom.rs` の残り55%(pucker_bloat / zigzag / offset / twist / wiggle)を取る |
| `layer-meta` | 16 | hd / parent / sr / bm / null-layer / ef / tm。store を触るので直列 |
| `effect` | 10 | 裁定70/72。Document は plugin id + param map だけ持つ |
| `mask` | 7 | **`Value::Path` / `Bool` は済**。残りは `helpers/mask` と `constants/mask-mode` |
| `slot` | 4 | Vism の先例になりうる |
| `marker` | 3 | 独立。ロケータに直接写る |
| `shape-3` | 3 | twist |
| `split-position` | 3 | 独立 |
| `motion-path` | 2 | 空間ベジェ。`Value::Path` に依存 |
| `transform-skew` | 2 | `LayerPlacement::from_transform` に穴は空けてある |

`shape-1` は **38/38 完了**。

---

## 6. 次の人が最初にやるべきこと(順序つき)

1. **`cd next && cargo test`(debug)を通す**。release だけで判断しない(裁定102)
2. **裁定98 を地図へ反映する** — `text-document f/s/fc/lh/tr/sc/sw/of` の note に「**スタイル表の既定行(index 0)**として読む」を書く。今は裁定文にしか無く、**地図の note が黙っている**
3. **`mask` を通す**(残り7)。store を触るので他と並列にしない
4. `marker` / `split-position` / `transform-skew`(計8)は独立なので続けて
5. **`shape-2` と `layer-meta` は並列に投げられる**(前者は `motolii-vector` 内で完結、後者は store)。ただし**同時に store を触る束を2つ投げない**
6. **実機で1度動かす** — `cargo run -p motolii-shell`。裁定1〜106 は全部これを通っていない

---

## 7. この日に自分で間違えたこと(再発防止のため残す)

| 誤り | どう露見したか |
|---|---|
| **engine で `t.num() as f64 * fps.num() as f64` と書いた** | `motolii-core` が「f64×fps の独自丸めは禁止」と明記しているのに。**旧 TM-4 の柵が5個の文字列 grep だったので素通り**。敵対的レビューが発見 |
| **`byte 一致試験` が同じ関数を2回呼ぶだけだった** | 変数名が `preview`/`export` なだけで、第二経路が生えても絶対に落ちない試験。同上 |
| **フレームキャッシュが無限に伸びていた** | 300フレームで298枚保持。9,000フレームなら約27GB。自分のドリフト監査で発見 |
| **`try_from_frame` が既に core にあるのに同名を足した** | 「読まずに再発明」をまさに実演。コンパイルエラーで露見 |
| **`--release` だけで走らせて `debug_assert` を黙らせていた** | 並列エージェントが debug で踏んで発覚 |
| **Rive 行を vendor せずに地図へ書いた** | 調査担当が「上流が動いたら黙ってずれる」と指摘 |

**共通する形**: 自分で書いた柵を自分がすり抜けている。**外の目(敵対的レビュー / 並列エージェント)が5件中4件を見つけた。**

---

## 8. 明示的に未着手・未決

- **選択と playhead の置き場**。undo の対象にすべきでないので `edit` timeline とは別扱いが要る。決めないまま shell を建てると shell 側に置かれ、そこが次の翻訳層になる
- **拡張の口(trait)**。裁定13 で「2人目の利用者が現れるまで作らない」。裁定72 で「param は既存の KeyframeTrack に乗る = 新機構ゼロ」まで分かっている
- **音声一式**(decode / mix / 再生 / export mux)。旧 `motolii-audio` 4,286行が移植候補
- **ラスタライズ解像度の正本**(裁定105)。shape だけ「後から解像度を選べる素材」という非対称
- **alpha 付き書き出し**(裁定16)。`ScreenshotProcessor` が composite 後を撮るので alpha が 255 に潰れる。**限界を試験で固定済み**なので直った日に落ちる
- **「seed 付き randomize」に先例が1つも無い**(裁定101)。自前設計が要る
- 未使用の重い依存(`re_video` / `tonic` / `prost` / `ffmpeg-sidecar`)。害は無いがビルド時間の無駄
